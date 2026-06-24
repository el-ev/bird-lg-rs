import { describe, expect, it } from "vitest";

import {
  claimValueAtPath,
  createOidcAuthorizationRequest,
  discoveryUrlForProvider,
  jwksUrlForProvider,
  oidcAsnFromClaimSources,
  oidcMaintainerFromClaimSources,
  rewriteIssuerHost,
} from "dn42-auth-worker/oidc";
import type { OidcProviderConfig } from "../src/types";

function provider(overrides: Partial<OidcProviderConfig> = {}): OidcProviderConfig {
  return {
    name: "example",
    label: "Example Auth",
    issuer: "https://issuer.example",
    client_id: "client-id",
    audience: "client-id",
    asn_claim: "asn",
    mntner_claim: "mntner",
    ...overrides,
  };
}

describe("OIDC discovery URL derivation", () => {
  it("uses an explicit discovery URL when present", () => {
    expect(
      discoveryUrlForProvider(
        provider({ discovery_url: "https://issuer.example/custom/discovery.json" }),
      ).href,
    ).toBe("https://issuer.example/custom/discovery.json");
  });

  it("uses the host-root discovery document for root issuers", () => {
    expect(discoveryUrlForProvider(provider()).href).toBe(
      "https://issuer.example/.well-known/openid-configuration",
    );
  });

  it("preserves issuer paths in the discovery URL", () => {
    expect(
      discoveryUrlForProvider(
        provider({ issuer: "https://issuer.example/realms/dn42" }),
      ).href,
    ).toBe("https://issuer.example/.well-known/openid-configuration/realms/dn42");
  });
});

describe("OIDC JWKS URL derivation", () => {
  it("keeps an explicit jwks_uri", () => {
    expect(
      jwksUrlForProvider(
        provider({ jwks_uri: "https://issuer.example/custom/keys.json" }),
      ).href,
    ).toBe("https://issuer.example/custom/keys.json");
  });

  it("falls back to discovery metadata when available", () => {
    expect(
      jwksUrlForProvider(provider(), {
        jwks_uri: "https://issuer.example/discovery/jwks.json",
      }).href,
    ).toBe("https://issuer.example/discovery/jwks.json");
  });
});

describe("OIDC authorization request generation", () => {
  it("builds a PKCE authorization URL with configured scopes", async () => {
    const { authorizationUrl, record } = await createOidcAuthorizationRequest(
      provider({
        scopes: ["openid", "profile", "email", "dn42"],
      }),
      {
        issuer: "https://issuer.example",
        authorization_endpoint: "https://issuer.example/oauth2/auth",
        token_endpoint: "https://issuer.example/oauth2/token",
        jwks_uri: "https://issuer.example/oauth2/jwks.json",
      },
      "challenge-1",
      "https://autopeer.example/oidc/callback/example",
    );

    const parsed = new URL(authorizationUrl);
    expect(parsed.origin + parsed.pathname).toBe("https://issuer.example/oauth2/auth");
    expect(parsed.searchParams.get("response_type")).toBe("code");
    expect(parsed.searchParams.get("client_id")).toBe("client-id");
    expect(parsed.searchParams.get("redirect_uri")).toBe(
      "https://autopeer.example/oidc/callback/example",
    );
    expect(parsed.searchParams.get("scope")).toBe("openid profile email dn42");
    expect(parsed.searchParams.get("code_challenge_method")).toBe("S256");
    expect(parsed.searchParams.get("state")).toBe(record.state);
    expect(parsed.searchParams.get("nonce")).toBe(record.nonce);
    expect(parsed.searchParams.get("code_challenge")).toBeTruthy();
    expect(record.challenge_id).toBe("challenge-1");
    expect(record.provider).toBe("example");
    expect(record.code_verifier).toBeTruthy();
  });
});

describe("OIDC claim paths", () => {
  it("supports nested Kioubit-style claim objects", () => {
    const payload = {
      dn42: {
        asn: "4242421234",
        mnt: ["IRIS-MNT", "EXTRA-MNT"],
      },
    };

    expect(claimValueAtPath(payload, "dn42.asn")).toBe("4242421234");
    expect(claimValueAtPath(payload, "dn42.mnt")).toEqual(["IRIS-MNT", "EXTRA-MNT"]);
  });

  it("supports fallback claim paths for iEdon-style payloads", () => {
    const claimSources = [
      {
        profile: {
          asn: 4242422589,
          active_mnt: "IEDON-MNT",
          mnt_by: ["IEDON-MNT"],
        },
      },
    ];

    expect(
      oidcAsnFromClaimSources(
        claimSources,
        provider({ asn_claim: ["dn42.asn", "profile.asn"] }),
      ),
    ).toBe("4242422589");
    expect(
      oidcMaintainerFromClaimSources(
        claimSources,
        provider({
          mntner_claim: ["profile.active_mnt", "profile.mnt_by", "dn42.mnt"],
        }),
        [
          {
            name: "IEDON-MNT",
            auth_lines: [],
            ssh_public_keys: [],
            ssh_fingerprints: [],
            pgp_fingerprints: [],
            contact_emails: [],
          },
        ],
      ),
    ).toBe("IEDON-MNT");
  });
});

describe("rewriteIssuerHost", () => {
  it("rewrites matching issuer host to dn42", () => {
    const p = provider({
      issuer: "https://dn42.g-load.eu",
      dn42_issuer: "https://auth.dn42",
    });
    expect(rewriteIssuerHost("https://dn42.g-load.eu/oauth/authorize", p)).toBe(
      "https://auth.dn42/oauth/authorize",
    );
  });

  it("returns url unchanged when no dn42_issuer", () => {
    const p = provider({ issuer: "https://dn42.g-load.eu" });
    expect(rewriteIssuerHost("https://dn42.g-load.eu/oauth/authorize", p)).toBe(
      "https://dn42.g-load.eu/oauth/authorize",
    );
  });

  it("returns url unchanged when host does not match issuer", () => {
    const p = provider({
      issuer: "https://dn42.g-load.eu",
      dn42_issuer: "https://auth.dn42",
    });
    expect(rewriteIssuerHost("https://other.example/authorize", p)).toBe(
      "https://other.example/authorize",
    );
  });
});
