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
import {
  HttpError,
  addSeconds,
  nowIso,
  requireNonEmptyString,
  uiMessage,
} from "./utils";

const CHALLENGE_TTL_SECONDS = 15 * 60;
const SESSION_TTL_SECONDS = 6 * 60 * 60;
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
    invalidSshSignature("error.auth.ssh.malformed_signature");
  }

  const view = new DataView(reader.bytes.buffer, reader.bytes.byteOffset + reader.offset, 4);
  const value = view.getUint32(0, false);
  reader.offset += 4;
  return value;
}

function readBytes(reader: Reader, length: number): Uint8Array {
  if (reader.offset + length > reader.bytes.length) {
    invalidSshSignature("error.auth.ssh.malformed_signature");
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
    invalidSshSignature("error.auth.ssh.empty_or_missing_blocks");
  }
  if (!trimmed.includes("-----BEGIN SSH SIGNATURE-----") || !trimmed.includes("-----END SSH SIGNATURE-----")) {
    if (trimmed.includes("dn42-autopeer challenge")) {
      invalidSshSignature("error.auth.ssh.unsigned_challenge");
    }
    invalidSshSignature("error.auth.ssh.empty_or_missing_blocks");
  }

  const base64 = signature
    .replace(/-----BEGIN SSH SIGNATURE-----/g, "")
    .replace(/-----END SSH SIGNATURE-----/g, "")
    .replace(/\s+/g, "");
  if (base64.length === 0) {
    invalidSshSignature("error.auth.ssh.empty_or_missing_blocks");
  }

  let binary: string;
  try {
    binary = atob(base64);
  } catch {
    invalidSshSignature("error.auth.ssh.malformed_signature");
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
    invalidSshSignature("error.auth.ssh.malformed_signature");
  }

  const version = readUint32(reader);
  if (version !== 1) {
    invalidSshSignature("error.auth.ssh.malformed_signature");
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
    invalidSshSignature("error.auth.ssh.malformed_signature");
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
    label: uiMessage("auth_method.registry_email.label"),
    description: uiMessage("auth_method.registry_email.session_description", {
      mnt: effectiveMnt,
    }),
  });
}

export function assertChallengeFresh(challenge: ChallengeRecord): void {
  if (Date.parse(challenge.expires_at) <= Date.now()) {
    throw new HttpError("error.auth.challenge.expired", 400);
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
    throw new HttpError("error.auth.ssh.unrecognized_key", 400);
  }

  const valid = await verifyAgainstChallenge(signature, challenge.challenge_text);
  if (!valid) {
    throw new HttpError("error.auth.ssh.verification_failed", 400);
  }

  return buildSessionRecord(challenge.asn, maintainer.name, {
    kind: "registry_ssh",
    label: uiMessage("auth_method.registry_ssh.label"),
    description: uiMessage("auth_method.registry_ssh.session_description", {
      mnt: maintainer.name,
    }),
  });
}

export async function verifyRegistryPgpChallenge(
  challenge: ChallengeRecord,
  request: RegistryPgpVerifyRequest,
): Promise<SessionRecord> {
  const armoredKey = requireAuthField(request.public_key, "public_key");
  const signedMessage = requireAuthField(request.signed_message, "signed_message");

  const publicKey = await readKey({ armoredKey }).catch(() =>
    invalidPgpSignature("error.auth.pgp.invalid_public_key"),
  );

  const fingerprint = publicKey.getFingerprint().toUpperCase();
  const maintainer = matchingMaintainerByFingerprint(challenge.maintainers, fingerprint);
  if (!maintainer) {
    throw new HttpError(
      uiMessage("error.auth.pgp.unrecognized_key", { fingerprint }),
      400,
    );
  }

  const cleartext = await readCleartextMessage({
    cleartextMessage: signedMessage,
  }).catch(() => invalidPgpSignature("error.auth.pgp.invalid_signed_message"));

  const verification = await verifyPgp({
      message: cleartext,
      verificationKeys: publicKey,
    }).catch(() => invalidPgpSignature("error.auth.pgp.verification_failed"));

  const signedText = cleartext.getText().trimEnd();
  if (signedText !== challenge.challenge_text.trimEnd()) {
    throw new HttpError("error.auth.pgp.challenge_mismatch", 400);
  }

  for (const signature of verification.signatures) {
    try {
      await signature.verified;
    } catch {
      invalidPgpSignature("error.auth.pgp.verification_failed");
    }
  }

  return buildSessionRecord(challenge.asn, maintainer.name, {
    kind: "registry_pgp",
    label: uiMessage("auth_method.registry_pgp.label"),
    description: uiMessage("auth_method.registry_pgp.session_description", {
      mnt: maintainer.name,
    }),
  });
}
