import { describe, expect, it } from "vitest";

import { resolveLocale, resolveLocaleCode, t } from "../src/i18n";

describe("worker i18n", () => {
  it("resolves German locale tags", () => {
    const request = new Request("https://worker.example", {
      headers: { "accept-language": "de-DE,de;q=0.9,en;q=0.8" },
    });

    expect(resolveLocale(request)).toBe("de");
    expect(resolveLocaleCode("de-CH")).toBe("de");
  });

  it("renders German email templates with parameters", () => {
    expect(t("de", "email.subject", { asn: "4242421024" })).toBe(
      "dn42 Autopeer-Login für AS4242421024",
    );
    expect(t("de", "email.expires", { expires_at: "2026-04-24T12:00:00Z" })).toBe(
      "Dieser Code läuft um 2026-04-24T12:00:00Z ab.",
    );
  });
});
