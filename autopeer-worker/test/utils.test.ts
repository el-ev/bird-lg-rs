import { describe, expect, it } from "vitest";

import {
  normalizeAsn,
  normalizeSupportedAutopeerAsn,
  requireBoolean,
  requireOptionalInteger,
  requireOptionalString,
  requireRecord,
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
    expect(stripOperatorHints("unknown challenge_id")).toBe("unknown challenge_id");
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
      expect(() => normalizeAsn(value)).toThrow("ASN must look like 424242xxxx");
    }
  });

  it("treats non-424242 ranges as unsupported for autopeer auth flows", () => {
    for (const value of ["4242431024", "AS64512", "foo4242421024"]) {
      expect(() => normalizeSupportedAutopeerAsn(value)).toThrow(
        "We do not support that ASN range yet. Right now Autopeer only supports 424242xxxx.",
      );
    }
  });
});

describe("request value validation", () => {
  it("treats blank optional strings as absent but rejects non-strings", () => {
    expect(requireOptionalString("  ", "field")).toBeNull();
    expect(requireOptionalString(" value ", "field")).toBe("value");
    expect(() => requireOptionalString(123, "field")).toThrow("field must be a string");
  });

  it("requires booleans, integers, and object records", () => {
    expect(requireBoolean(true, "flag")).toBe(true);
    expect(requireOptionalInteger(42, "count")).toBe(42);
    expect(requireOptionalInteger(null, "count")).toBeNull();
    expect(requireRecord({ ok: true }, "payload")).toEqual({ ok: true });

    expect(() => requireBoolean("true", "flag")).toThrow("flag must be a boolean");
    expect(() => requireOptionalInteger(1.5, "count")).toThrow("count must be an integer");
    expect(() => requireRecord([], "payload")).toThrow("payload must be an object");
  });
});
