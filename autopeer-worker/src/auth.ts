import { readCleartextMessage, readKey, verify as verifyPgp } from "openpgp";
import { verify as verifySshSignature } from "sshsig";

import type {
  AuthMethod,
  ChallengeRecord,
  MaintainerRecord,
  RegistryEmailAuthRequestRecord,
  RegistryPgpVerifyRequest,
  RegistrySshVerifyRequest,
  SessionRecord,
} from "./types";
import { HttpError, addSeconds, nowIso, requireNonEmptyString } from "./utils";

const CHALLENGE_TTL_SECONDS = 15 * 60;
const SESSION_TTL_SECONDS = 6 * 60 * 60;
const SSH_SIGNATURE_ARMOR_HINT =
  "Paste the full detached SSH signature block produced by ssh-keygen -Y sign, including the BEGIN/END lines.";
const SSH_SIGNATURE_CHALLENGE_HINT =
  "Paste the detached SSH signature block produced by ssh-keygen -Y sign, not the unsigned challenge text.";
const SSH_SIGNATURE_MALFORMED_HINT =
  "SSH signature data is malformed. Re-run ssh-keygen -Y sign and paste the full detached signature block.";
const PGP_PUBLIC_KEY_HINT =
  "PGP public key is invalid. Export your ASCII-armored public key and paste the full block.";
const PGP_SIGNED_MESSAGE_HINT =
  "PGP signed message is invalid. Clear-sign the challenge and paste the full signed block.";
const PGP_VERIFICATION_HINT =
  "PGP signature verification failed. Re-sign the challenge with the matching registry key and paste the full signed block.";
const EMAIL_AUTH_TTL_SECONDS = 15 * 60;

type Reader = {
  bytes: Uint8Array;
  offset: number;
};

function invalidSshSignature(message: string): never {
  throw new HttpError(message, 400);
}

function invalidPgpSignature(message: string): never {
  throw new HttpError(message, 400);
}

function requireAuthField(value: unknown, field: string): string {
  try {
    return requireNonEmptyString(value, field);
  } catch (error) {
    throw new HttpError(error instanceof Error ? error.message : `${field} is required`, 400);
  }
}

function readUint32(reader: Reader): number {
  if (reader.offset + 4 > reader.bytes.length) {
    invalidSshSignature(SSH_SIGNATURE_MALFORMED_HINT);
  }

  const view = new DataView(reader.bytes.buffer, reader.bytes.byteOffset + reader.offset, 4);
  const value = view.getUint32(0, false);
  reader.offset += 4;
  return value;
}

function readBytes(reader: Reader, length: number): Uint8Array {
  if (reader.offset + length > reader.bytes.length) {
    invalidSshSignature(SSH_SIGNATURE_MALFORMED_HINT);
  }

  const slice = reader.bytes.slice(reader.offset, reader.offset + length);
  reader.offset += length;
  return slice;
}

function readStringBytes(reader: Reader): Uint8Array {
  return readBytes(reader, readUint32(reader));
}

function bytesToString(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes);
}

function armorToBytes(signature: string): Uint8Array {
  const trimmed = signature.trim();
  if (trimmed.length === 0) {
    invalidSshSignature(SSH_SIGNATURE_ARMOR_HINT);
  }
  if (!trimmed.includes("-----BEGIN SSH SIGNATURE-----") || !trimmed.includes("-----END SSH SIGNATURE-----")) {
    if (trimmed.includes("dn42-autopeer challenge")) {
      invalidSshSignature(SSH_SIGNATURE_CHALLENGE_HINT);
    }
    invalidSshSignature(SSH_SIGNATURE_ARMOR_HINT);
  }

  const base64 = signature
    .replace(/-----BEGIN SSH SIGNATURE-----/g, "")
    .replace(/-----END SSH SIGNATURE-----/g, "")
    .replace(/\s+/g, "");
  if (base64.length === 0) {
    invalidSshSignature(SSH_SIGNATURE_ARMOR_HINT);
  }

  let binary: string;
  try {
    binary = atob(base64);
  } catch {
    invalidSshSignature(SSH_SIGNATURE_MALFORMED_HINT);
  }
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function normalizeSshPublicKey(publicKey: string): string {
  return publicKey.trim().split(/\s+/).slice(0, 2).join(" ");
}

function parseSshSignature(signature: string): { publicKey: string } {
  const bytes = armorToBytes(signature);
  const reader: Reader = { bytes, offset: 0 };

  const magic = bytesToString(readBytes(reader, 6));
  if (magic !== "SSHSIG") {
    invalidSshSignature(SSH_SIGNATURE_MALFORMED_HINT);
  }

  const version = readUint32(reader);
  if (version !== 1) {
    invalidSshSignature(SSH_SIGNATURE_MALFORMED_HINT);
  }

  const rawPublicKey = readStringBytes(reader);
  const publicKeyReader: Reader = { bytes: rawPublicKey, offset: 0 };
  const algorithm = bytesToString(readStringBytes(publicKeyReader));
  const publicKey = `${algorithm} ${btoa(String.fromCharCode(...rawPublicKey))}`;

  return {
    publicKey,
  };
}

function challengePayload(asn: string, challengeId: string, issuedAt: string): string {
  return [
    "dn42-autopeer challenge",
    `asn: ${asn}`,
    `challenge_id: ${challengeId}`,
    `issued_at: ${issuedAt}`,
  ].join("\n");
}

function buildSessionRecord(asn: string, effectiveMnt: string, authMethod: AuthMethod): SessionRecord {
  const createdAt = nowIso();
  return {
    token: crypto.randomUUID(),
    asn,
    effective_mnt: effectiveMnt,
    auth_method: authMethod,
    created_at: createdAt,
    expires_at: addSeconds(createdAt, SESSION_TTL_SECONDS),
  };
}

function randomDigits(length: number): string {
  const bytes = crypto.getRandomValues(new Uint8Array(length));
  return Array.from(bytes, (byte) => String(byte % 10)).join("");
}

function randomBase64Url(length = 32): string {
  const bytes = crypto.getRandomValues(new Uint8Array(length));
  const binary = Array.from(bytes, (byte) => String.fromCharCode(byte)).join("");
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/u, "");
}

function matchingMaintainerBySshKey(
  maintainers: MaintainerRecord[],
  publicKey: string,
): MaintainerRecord | undefined {
  const normalized = normalizeSshPublicKey(publicKey);
  return maintainers.find((maintainer) =>
    maintainer.ssh_public_keys.some((candidate) => normalizeSshPublicKey(candidate) === normalized),
  );
}

function matchingMaintainerByFingerprint(
  maintainers: MaintainerRecord[],
  fingerprint: string,
): MaintainerRecord | undefined {
  const normalized = fingerprint.toUpperCase().replace(/\s+/g, "");
  return maintainers.find((maintainer) =>
    maintainer.pgp_fingerprints.some((candidate) => candidate === normalized),
  );
}

async function verifyAgainstChallenge(signature: string, challengeText: string): Promise<boolean> {
  try {
    if (await verifySshSignature(signature, challengeText)) {
      return true;
    }
    return challengeText.endsWith("\n")
      ? false
      : await verifySshSignature(signature, `${challengeText}\n`);
  } catch {
    invalidSshSignature(SSH_SIGNATURE_MALFORMED_HINT);
  }
}

export function createChallenge(asn: string): ChallengeRecord {
  const createdAt = nowIso();
  const id = crypto.randomUUID();
  return {
    id,
    asn,
    challenge_text: challengePayload(asn, id, createdAt),
    methods: [],
    maintainers: [],
    created_at: createdAt,
    expires_at: addSeconds(createdAt, CHALLENGE_TTL_SECONDS),
  };
}

export function createRegistryEmailAuthRequest(
  challenge: ChallengeRecord,
  effectiveMnt: string,
  emails: string[],
): RegistryEmailAuthRequestRecord {
  const createdAt = nowIso();
  return {
    challenge_id: challenge.id,
    effective_mnt: effectiveMnt,
    email_snapshot: [...new Set(emails)],
    code: randomDigits(8),
    token: randomBase64Url(32),
    session_token: null,
    created_at: createdAt,
    expires_at: addSeconds(createdAt, EMAIL_AUTH_TTL_SECONDS),
  };
}

export function createRegistryEmailSession(
  challenge: ChallengeRecord,
  effectiveMnt: string,
): SessionRecord {
  return buildSessionRecord(challenge.asn, effectiveMnt, {
    kind: "registry_email",
    label: "Registry Email Magic Link",
    description: `You authenticated with ${effectiveMnt} using registry email auth.`,
  });
}

export function assertChallengeFresh(challenge: ChallengeRecord): void {
  if (Date.parse(challenge.expires_at) <= Date.now()) {
    throw new HttpError("challenge has expired", 400);
  }
}

export async function verifyRegistrySshChallenge(
  challenge: ChallengeRecord,
  request: RegistrySshVerifyRequest,
): Promise<SessionRecord> {
  const signature = requireAuthField(request.signature, "signature");
  const parsed = parseSshSignature(signature);

  const maintainer = matchingMaintainerBySshKey(challenge.maintainers, parsed.publicKey);
  if (!maintainer) {
    throw new HttpError(
      "Your SSH signature used a key that is not present in the resolved maintainer objects",
      400,
    );
  }

  const valid = await verifyAgainstChallenge(signature, challenge.challenge_text);
  if (!valid) {
    throw new HttpError("SSH signature verification failed", 400);
  }

  return buildSessionRecord(challenge.asn, maintainer.name, {
    kind: "registry_ssh",
    label: "Registry SSH Signature",
    description: `You authenticated with ${maintainer.name} using registry SSH auth.`,
  });
}

export async function verifyRegistryPgpChallenge(
  challenge: ChallengeRecord,
  request: RegistryPgpVerifyRequest,
): Promise<SessionRecord> {
  const armoredKey = requireAuthField(request.public_key, "public_key");
  const signedMessage = requireAuthField(request.signed_message, "signed_message");

  const publicKey = await readKey({ armoredKey }).catch(() =>
    invalidPgpSignature(PGP_PUBLIC_KEY_HINT),
  );

  const fingerprint = publicKey.getFingerprint().toUpperCase();
  const maintainer = matchingMaintainerByFingerprint(challenge.maintainers, fingerprint);
  if (!maintainer) {
    throw new HttpError(
      `Your PGP fingerprint ${fingerprint} is not present in the resolved maintainer objects`,
      400,
    );
  }

  const cleartext = await readCleartextMessage({
    cleartextMessage: signedMessage,
  }).catch(() => invalidPgpSignature(PGP_SIGNED_MESSAGE_HINT));

  const verification = await verifyPgp({
      message: cleartext,
      verificationKeys: publicKey,
    }).catch(() => invalidPgpSignature(PGP_VERIFICATION_HINT));

  const signedText = cleartext.getText().trimEnd();
  if (signedText !== challenge.challenge_text.trimEnd()) {
    throw new HttpError("Your PGP signed message does not match the issued challenge", 400);
  }

  for (const signature of verification.signatures) {
    try {
      await signature.verified;
    } catch {
      invalidPgpSignature(PGP_VERIFICATION_HINT);
    }
  }

  return buildSessionRecord(challenge.asn, maintainer.name, {
    kind: "registry_pgp",
    label: "Registry PGP Signature",
    description: `You authenticated with ${maintainer.name} using registry PGP auth.`,
  });
}
