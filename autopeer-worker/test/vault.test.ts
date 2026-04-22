import { describe, expect, it } from "vitest";
import { vaultEncrypt, vaultDecrypt, isVaultEncrypted } from "../src/vault";

describe("vault", () => {
  const password = "test-vault-password";

  it("roundtrips encrypt then decrypt", async () => {
    const plaintext = "peer.example.net:21023";
    const encrypted = await vaultEncrypt(plaintext, password);
    expect(encrypted.tag).toBe("!vault");
    expect(encrypted.value).toMatch(/^\$ANSIBLE_VAULT;1\.1;AES256\n/);
    const decrypted = await vaultDecrypt(encrypted, password);
    expect(decrypted).toBe(plaintext);
  });

  it("rejects wrong password", async () => {
    const encrypted = await vaultEncrypt("secret", password);
    await expect(vaultDecrypt(encrypted, "wrong-password")).rejects.toThrow(
      /HMAC verification failed/,
    );
  });

  it("roundtrips empty plaintext", async () => {
    const encrypted = await vaultEncrypt("", password);
    const decrypted = await vaultDecrypt(encrypted, password);
    expect(decrypted).toBe("");
  });

  it("isVaultEncrypted identifies tagged scalars", () => {
    expect(isVaultEncrypted({ tag: "!vault", value: "anything" })).toBe(true);
    expect(isVaultEncrypted({ tag: "!other", value: "x" })).toBe(false);
    expect(isVaultEncrypted("plain string")).toBe(false);
    expect(isVaultEncrypted(null)).toBe(false);
  });
});
