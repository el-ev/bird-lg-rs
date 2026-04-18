import { describe, expect, it } from "vitest";
import {
  createCleartextMessage,
  generateKey,
  readKey,
  readPrivateKey,
  sign,
} from "openpgp";

import { verifyRegistryPgpChallenge, verifyRegistrySshChallenge } from "../src/auth";
import type { ChallengeRecord, MaintainerRecord } from "../src/types";

function challengeRecord(): ChallengeRecord {
  return {
    id: "challenge-1",
    asn: "4242421024",
    challenge_text: [
      "dn42-autopeer challenge",
      "asn: 4242421024",
      "challenge_id: challenge-1",
      "issued_at: 2026-04-18T12:42:04.075Z",
    ].join("\n"),
    methods: [],
    maintainers: [],
    created_at: "2026-04-18T12:42:04.075Z",
    expires_at: "2026-04-18T13:42:04.075Z",
  };
}

type GeneratedPgpIdentity = {
  fingerprint: string;
  privateKey: string;
  publicKey: string;
};

let generatedPgpIdentityPromise: Promise<GeneratedPgpIdentity> | undefined;

async function generatedPgpIdentity(): Promise<GeneratedPgpIdentity> {
  generatedPgpIdentityPromise ??= (async () => {
    const { privateKey, publicKey } = await generateKey({
      type: "ecc" as const,
      curve: "ed25519Legacy" as const,
      userIDs: [{ name: "Autopeer Test" }],
      format: "armored",
    });
    const parsedPublicKey = await readKey({ armoredKey: publicKey });
    return {
      fingerprint: parsedPublicKey.getFingerprint().toUpperCase(),
      privateKey,
      publicKey,
    };
  })();

  return generatedPgpIdentityPromise;
}

async function clearsignedMessage(text: string): Promise<string> {
  const identity = await generatedPgpIdentity();
  const signingKey = await readPrivateKey({ armoredKey: identity.privateKey });
  const message = await createCleartextMessage({ text });
  return sign({
    message,
    signingKeys: signingKey,
    format: "armored",
  });
}

function pgpMaintainer(fingerprint: string): MaintainerRecord {
  return {
    name: "TEST-MNT",
    auth_lines: [`pgp-fingerprint ${fingerprint}`],
    ssh_public_keys: [],
    ssh_fingerprints: [],
    pgp_fingerprints: [fingerprint],
    contact_emails: [],
  };
}

describe("registry SSH auth validation", () => {
  it("rejects unsigned challenge text with a user-facing hint", async () => {
    const challenge = challengeRecord();

    await expect(
      verifyRegistrySshChallenge(challenge, {
        challenge_id: challenge.id,
        signature: challenge.challenge_text,
      }),
    ).rejects.toMatchObject({
      message:
        "Paste the detached SSH signature block produced by ssh-keygen -Y sign, not the unsigned challenge text.",
      status: 400,
    });
  });

  it("rejects malformed SSH armor without leaking atob internals", async () => {
    const challenge = challengeRecord();

    await expect(
      verifyRegistrySshChallenge(challenge, {
        challenge_id: challenge.id,
        signature: "-----BEGIN SSH SIGNATURE-----\nnot-base64!!\n-----END SSH SIGNATURE-----",
      }),
    ).rejects.toMatchObject({
      message:
        "SSH signature data is malformed. Re-run ssh-keygen -Y sign and paste the full detached signature block.",
      status: 400,
    });
  });

  it("rejects non-string SSH signatures as client errors", async () => {
    const challenge = challengeRecord();

    await expect(
      verifyRegistrySshChallenge(challenge, {
        challenge_id: challenge.id,
        signature: {} as never,
      }),
    ).rejects.toMatchObject({
      message: "signature is required",
      status: 400,
    });
  });
});

describe("registry PGP auth validation", () => {
  it("rejects malformed public keys as client errors", async () => {
    const challenge = challengeRecord();

    await expect(
      verifyRegistryPgpChallenge(challenge, {
        challenge_id: challenge.id,
        public_key: "-----BEGIN PGP PUBLIC KEY BLOCK-----\nnot-a-key\n-----END PGP PUBLIC KEY BLOCK-----",
        signed_message: "unused",
      }),
    ).rejects.toMatchObject({
      message:
        "PGP public key is invalid. Export your ASCII-armored public key and paste the full block.",
      status: 400,
    });
  });

  it("rejects fingerprints that are not present in the resolved maintainer objects", async () => {
    const challenge = challengeRecord();
    const identity = await generatedPgpIdentity();

    challenge.maintainers = [pgpMaintainer("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")];

    await expect(
      verifyRegistryPgpChallenge(challenge, {
        challenge_id: challenge.id,
        public_key: identity.publicKey,
        signed_message: await clearsignedMessage(challenge.challenge_text),
      }),
    ).rejects.toMatchObject({
      message: `Your PGP fingerprint ${identity.fingerprint} is not present in the resolved maintainer objects`,
      status: 400,
    });
  });

  it("rejects malformed clear-signed payloads as client errors", async () => {
    const challenge = challengeRecord();
    const identity = await generatedPgpIdentity();

    challenge.maintainers = [pgpMaintainer(identity.fingerprint)];

    await expect(
      verifyRegistryPgpChallenge(challenge, {
        challenge_id: challenge.id,
        public_key: identity.publicKey,
        signed_message: "not a clearsigned message",
      }),
    ).rejects.toMatchObject({
      message:
        "PGP signed message is invalid. Clear-sign the challenge and paste the full signed block.",
      status: 400,
    });
  });

  it("rejects non-string PGP payload fields as client errors", async () => {
    const challenge = challengeRecord();

    await expect(
      verifyRegistryPgpChallenge(challenge, {
        challenge_id: challenge.id,
        public_key: {} as never,
        signed_message: [] as never,
      }),
    ).rejects.toMatchObject({
      message: "public_key is required",
      status: 400,
    });
  });

  it("rejects clear-signed messages that do not match the issued challenge", async () => {
    const challenge = challengeRecord();
    const identity = await generatedPgpIdentity();

    challenge.maintainers = [pgpMaintainer(identity.fingerprint)];

    await expect(
      verifyRegistryPgpChallenge(challenge, {
        challenge_id: challenge.id,
        public_key: identity.publicKey,
        signed_message: await clearsignedMessage(`${challenge.challenge_text}\nextra line`),
      }),
    ).rejects.toMatchObject({
      message: "Your PGP signed message does not match the issued challenge",
      status: 400,
    });
  });
});
