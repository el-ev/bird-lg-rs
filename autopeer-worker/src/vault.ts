import type { TaggedScalar } from "./network.js";

const VAULT_HEADER = "$ANSIBLE_VAULT;1.1;AES256";
const PBKDF2_ITERATIONS = 10000;
const KEY_LENGTH = 32;
const IV_LENGTH = 16;
const DERIVED_KEY_LENGTH = 2 * KEY_LENGTH + IV_LENGTH;
const AES_BLOCK_SIZE = 16;
const HEX_LINE_WIDTH = 80;

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

function pkcs7Pad(data: Uint8Array): Uint8Array {
  const padding = AES_BLOCK_SIZE - (data.length % AES_BLOCK_SIZE) || AES_BLOCK_SIZE;
  const padded = new Uint8Array(data.length + padding);
  padded.set(data);
  padded.fill(padding, data.length);
  return padded;
}

function pkcs7Unpad(data: Uint8Array): Uint8Array {
  if (data.length === 0) {
    throw new Error("Cannot unpad empty data");
  }
  const padding = data[data.length - 1];
  if (padding < 1 || padding > AES_BLOCK_SIZE) {
    throw new Error("Invalid PKCS7 padding");
  }
  for (let i = data.length - padding; i < data.length; i++) {
    if (data[i] !== padding) {
      throw new Error("Invalid PKCS7 padding");
    }
  }
  return data.slice(0, data.length - padding);
}

async function deriveKeyMaterial(
  password: string,
  salt: Uint8Array,
): Promise<{ aesKey: CryptoKey; hmacKey: CryptoKey; iv: Uint8Array }> {
  const passwordKey = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(password),
    "PBKDF2",
    false,
    ["deriveBits"],
  );

  const derived = new Uint8Array(
    await crypto.subtle.deriveBits(
      { name: "PBKDF2", salt: salt as BufferSource, iterations: PBKDF2_ITERATIONS, hash: "SHA-256" },
      passwordKey,
      DERIVED_KEY_LENGTH * 8,
    ),
  );

  const aesKey = await crypto.subtle.importKey(
    "raw",
    derived.slice(0, KEY_LENGTH),
    { name: "AES-CTR" },
    false,
    ["encrypt", "decrypt"],
  );

  const hmacKey = await crypto.subtle.importKey(
    "raw",
    derived.slice(KEY_LENGTH, 2 * KEY_LENGTH),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"],
  );

  return { aesKey, hmacKey, iv: derived.slice(2 * KEY_LENGTH) };
}

export async function vaultEncrypt(
  plaintext: string,
  password: string,
): Promise<TaggedScalar> {
  const encoder = new TextEncoder();
  const salt = crypto.getRandomValues(new Uint8Array(32));
  const { aesKey, hmacKey, iv } = await deriveKeyMaterial(password, salt);

  const ciphertext = new Uint8Array(
    await crypto.subtle.encrypt(
      { name: "AES-CTR", counter: iv as BufferSource, length: 128 },
      aesKey,
      pkcs7Pad(encoder.encode(plaintext)) as BufferSource,
    ),
  );

  const hmac = new Uint8Array(await crypto.subtle.sign("HMAC", hmacKey, ciphertext as BufferSource));

  const inner = `${toHex(salt)}\n${toHex(hmac)}\n${toHex(ciphertext)}`;
  const outerHex = toHex(encoder.encode(inner));
  const lines: string[] = [];
  for (let i = 0; i < outerHex.length; i += HEX_LINE_WIDTH) {
    lines.push(outerHex.slice(i, i + HEX_LINE_WIDTH));
  }

  return { tag: "!vault", value: `${VAULT_HEADER}\n${lines.join("\n")}` };
}

export async function vaultDecrypt(
  tagged: TaggedScalar,
  password: string,
): Promise<string> {
  if (tagged.tag !== "!vault") {
    throw new Error("Not a vault-encrypted value");
  }

  const lines = tagged.value.trim().split("\n");
  if (lines[0] !== VAULT_HEADER) {
    throw new Error(`Unsupported vault format: ${lines[0]}`);
  }

  const inner = new TextDecoder().decode(fromHex(lines.slice(1).join("")));
  const [hexSalt, hexHmac, hexCiphertext] = inner.split("\n");
  const salt = fromHex(hexSalt);
  const expectedHmac = fromHex(hexHmac);
  const ciphertext = fromHex(hexCiphertext);

  const { aesKey, hmacKey, iv } = await deriveKeyMaterial(password, salt);

  const valid = await crypto.subtle.verify("HMAC", hmacKey, expectedHmac as BufferSource, ciphertext as BufferSource);
  if (!valid) {
    throw new Error("Vault HMAC verification failed (wrong password?)");
  }

  const decrypted = new Uint8Array(
    await crypto.subtle.decrypt(
      { name: "AES-CTR", counter: iv as BufferSource, length: 128 },
      aesKey,
      ciphertext as BufferSource,
    ),
  );

  return new TextDecoder().decode(pkcs7Unpad(decrypted));
}

export function isVaultEncrypted(value: unknown): value is TaggedScalar {
  return (
    typeof value === "object" &&
    value !== null &&
    "tag" in value &&
    "value" in value &&
    (value as TaggedScalar).tag === "!vault"
  );
}
