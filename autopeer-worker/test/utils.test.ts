import { describe, expect, it } from "vitest";

import {
  normalizeAsn,
  normalizeSupportedAutopeerAsn,
  stripOperatorHints,
} from "../src/utils";

describe("operator hint sanitization", () => {
  it("removes GitHub operator hints from public error messages", () => {
    const message =
      "GitHub file read failed for inventory.yaml: HTTP 403: Request forbidden by administrative rules. Hint: GitHub requires a valid User-Agent header on all API requests. GITHUB_TOKEN must also have the repository permissions required by this API call.";

    expect(stripOperatorHints(message)).toBe(
      "GitHub file read failed for inventory.yaml: HTTP 403: Request forbidden by administrative rules.",
    );
  });

  it("leaves normal user-facing errors untouched", () => {
    expect(stripOperatorHints("error.auth.challenge.unknown_id")).toBe("error.auth.challenge.unknown_id");
  });
});

describe("ASN normalization", () => {
  it("accepts DN42 ASNs with or without the AS prefix", () => {
    expect(normalizeAsn("4242421024")).toBe("4242421024");
    expect(normalizeAsn("AS4242421024")).toBe("4242421024");
    expect(normalizeSupportedAutopeerAsn("4242421024")).toBe("4242421024");
  });

  it("rejects malformed or non-DN42 identifiers", () => {
    for (const value of ["foo4242421024", "1234242421024", "1111111024", "4242431024"]) {
      expect(() => normalizeAsn(value)).toThrow("error.asn.format");
    }
  });

  it("treats non-424242 ranges as unsupported for autopeer auth flows", () => {
    for (const value of ["4242431024", "AS64512", "foo4242421024"]) {
      expect(() => normalizeSupportedAutopeerAsn(value)).toThrow("error.auth.asn.unsupported");
    }
  });
});

