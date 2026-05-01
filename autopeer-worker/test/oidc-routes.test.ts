import { beforeEach, describe, expect, it, vi } from "vitest";

import { UNSUPPORTED_ASN_RANGE_MESSAGE } from "../src/utils";
import { uiMessage } from "../src/utils";
import type {
  OidcAuthRequestRecord,
  OidcProviderConfig,
  OidcProviderDiscovery,
  SessionRecord,
} from "../src/types";

const dbMocks = vi.hoisted(() => ({
  getOidcAuthRequest: vi.fn(),
  putOidcAuthRequest: vi.fn(),
  deleteOidcAuthRequest: vi.fn(),
  getAuthSession: vi.fn(),
}));

const oidcMocks = vi.hoisted(() => ({
  fetchOidcDiscovery: vi.fn(),
  createOidcAuthorizationRequest: vi.fn(),
  exchangeAuthorizationCode: vi.fn(),
  verifiedOidcClaimSources: vi.fn(),
  oidcAsnFromClaimSources: vi.fn(),
}));

vi.mock("../src/db", async () => {
  const actual = await vi.importActual<typeof import("../src/db")>("../src/db");
  return {
    ...actual,
    getOidcAuthRequest: dbMocks.getOidcAuthRequest,
    putOidcAuthRequest: dbMocks.putOidcAuthRequest,
    deleteOidcAuthRequest: dbMocks.deleteOidcAuthRequest,
    getAuthSession: dbMocks.getAuthSession,
  };
});

vi.mock("../src/oidc", async () => {
  const actual = await vi.importActual<typeof import("../src/oidc")>("../src/oidc");
  return {
    ...actual,
    fetchOidcDiscovery: oidcMocks.fetchOidcDiscovery,
    createOidcAuthorizationRequest: oidcMocks.createOidcAuthorizationRequest,
    exchangeAuthorizationCode: oidcMocks.exchangeAuthorizationCode,
    verifiedOidcClaimSources: oidcMocks.verifiedOidcClaimSources,
    oidcAsnFromClaimSources: oidcMocks.oidcAsnFromClaimSources,
  };
});

import worker from "../src/index";

const provider: OidcProviderConfig = {
  name: "kioubit",
  label: "Kioubit",
  issuer: "https://issuer.example",
  client_id: "client-id",
  audience: "client-id",
  asn_claim: "dn42.asn",
  mntner_claim: "dn42.mnt",
};

const discovery: OidcProviderDiscovery = {
  issuer: provider.issuer,
  authorization_endpoint: "https://issuer.example/oauth2/auth",
  token_endpoint: "https://issuer.example/oauth2/token",
  jwks_uri: "https://issuer.example/oauth2/jwks.json",
};

function makeEnv(overrides: Partial<Env> = {}): Env {
  return {
    DB: {} as D1Database,
    GITHUB_OWNER: "owner",
    GITHUB_REPO: "repo",
    GITHUB_BASE_BRANCH: "main",
    DN42_REGISTRY_OWNER: "dn42",
    DN42_REGISTRY_REPO: "registry",
    DN42_REGISTRY_BRANCH: "master",
    DN42_REGISTRY_BASE_URL: "https://git.dn42.dev",
    OIDC_PROVIDERS: JSON.stringify([provider]),
    HOST_ASNS: "4242421023",
    AUTOPEER_API_URL: "https://api.autopeer.example",
    AUTOPEER_SITE_URL: "https://autopeer.example",
    AUTOPEER_TRUSTED_FORWARDED_HOSTS: "autopeer.iris.dn42",
    LOOKING_GLASS_URL: "https://lg.example",
    ...overrides,
  } as unknown as Env;
}

function jsonRequest(path: string, body: unknown): Request {
  return new Request(`https://worker.example${path}`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
}

function proxiedJsonRequest(path: string, body: unknown): Request {
  return new Request(`https://worker.example${path}`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-forwarded-host": "autopeer.iris.dn42",
      "x-forwarded-proto": "https",
    },
    body: JSON.stringify(body),
  });
}

function proxiedGetRequest(path: string): Request {
  return new Request(`https://worker.example${path}`, {
    headers: {
      "x-forwarded-host": "autopeer.iris.dn42",
      "x-forwarded-proto": "https",
    },
  });
}

function runWorker(request: Request, env = makeEnv()): Promise<Response> {
  return worker.fetch(request as never, env);
}

function authRequest(
  overrides: Partial<OidcAuthRequestRecord> = {},
): OidcAuthRequestRecord {
  return {
    state: "state-1",
    challenge_id: "",
    provider: "kioubit",
    nonce: "nonce-1",
    code_verifier: "code-verifier-1",
    redirect_uri: "https://autopeer.example/oidc/callback/kioubit",
    session_token: null,
    created_at: "2026-04-18T12:00:00.000Z",
    expires_at: "2999-01-01T00:00:00.000Z",
    ...overrides,
  };
}

function sessionRecord(overrides: Partial<SessionRecord> = {}): SessionRecord {
  return {
    token: "session-1",
    asn: "4242421024",
    effective_mnt: "IRIS-MNT",
    auth_method: {
      kind: "oidc",
      label: uiMessage("Kioubit"),
      description: uiMessage("auth_method.oidc.session_description", {
        provider: "Kioubit",
        mnt: "IRIS-MNT",
      }),
      provider: "kioubit",
    },
    created_at: "2026-04-18T12:00:00.000Z",
    expires_at: "2999-01-01T06:00:00.000Z",
    ...overrides,
  };
}

describe("OIDC worker routes", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("starts a challenge-less OIDC login and stores the state record", async () => {
    const record = authRequest();
    oidcMocks.fetchOidcDiscovery.mockResolvedValue(discovery);
    oidcMocks.createOidcAuthorizationRequest.mockResolvedValue({
      authorizationUrl: "https://issuer.example/oauth2/auth?state=state-1",
      record,
    });

    const response = await runWorker(jsonRequest("/v1/auth/oidc/kioubit/start", {}), makeEnv());

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      authorization_url: "https://issuer.example/oauth2/auth?state=state-1",
    });
    expect(oidcMocks.createOidcAuthorizationRequest).toHaveBeenCalledWith(
      provider,
      discovery,
      "",
      "https://autopeer.example/oidc/callback/kioubit",
    );
    expect(dbMocks.putOidcAuthRequest).toHaveBeenCalledWith(expect.anything(), record);
  });

  it("uses forwarded host headers for proxied OIDC callback urls", async () => {
    const record = authRequest({
      redirect_uri: "https://autopeer.iris.dn42/oidc/callback/kioubit",
    });
    oidcMocks.fetchOidcDiscovery.mockResolvedValue(discovery);
    oidcMocks.createOidcAuthorizationRequest.mockResolvedValue({
      authorizationUrl: "https://issuer.example/oauth2/auth?state=state-1",
      record,
    });

    const response = await runWorker(
      proxiedJsonRequest("/v1/auth/oidc/kioubit/start", {}),
      makeEnv(),
    );

    expect(response.status).toBe(200);
    expect(oidcMocks.createOidcAuthorizationRequest).toHaveBeenCalledWith(
      provider,
      discovery,
      "",
      "https://autopeer.iris.dn42/oidc/callback/kioubit",
    );
  });

  it("treats a repeated callback after successful login as idempotent", async () => {
    dbMocks.getOidcAuthRequest.mockResolvedValue(
      authRequest({ session_token: "session-1" }),
    );

    const response = await runWorker(
      new Request("https://worker.example/oidc/callback/kioubit?state=state-1&code=abc"),
      makeEnv(),
    );

    expect(response.status).toBe(302);
    expect(response.headers.get("location")).toBe(
      "https://autopeer.example/#oidc_state=state-1",
    );
    expect(oidcMocks.exchangeAuthorizationCode).not.toHaveBeenCalled();
  });

  it("redirects repeated proxied callbacks back to the forwarded site", async () => {
    dbMocks.getOidcAuthRequest.mockResolvedValue(
      authRequest({ session_token: "session-1" }),
    );

    const response = await runWorker(
      proxiedGetRequest("/oidc/callback/kioubit?state=state-1&code=abc"),
      makeEnv(),
    );

    expect(response.status).toBe(302);
    expect(response.headers.get("location")).toBe(
      "https://autopeer.iris.dn42/#oidc_state=state-1",
    );
  });

  it("rejects unsupported ASN claims during challenge-less callback redemption", async () => {
    dbMocks.getOidcAuthRequest.mockResolvedValue(authRequest());
    oidcMocks.fetchOidcDiscovery.mockResolvedValue(discovery);
    oidcMocks.exchangeAuthorizationCode.mockResolvedValue({ id_token: "id-token" });
    oidcMocks.verifiedOidcClaimSources.mockResolvedValue([{ dn42: { asn: "1111111234" } }]);
    oidcMocks.oidcAsnFromClaimSources.mockReturnValue("1111111234");

    const response = await runWorker(
      new Request("https://worker.example/oidc/callback/kioubit?state=state-1&code=abc"),
      makeEnv(),
    );

    expect(response.status).toBe(302);
    const location = new URL(response.headers.get("location")!);
    const error = JSON.parse(
      new URLSearchParams(location.hash.slice(1)).get("oidc_error")!,
    );
    expect(error).toEqual(uiMessage(UNSUPPORTED_ASN_RANGE_MESSAGE));
    expect(dbMocks.deleteOidcAuthRequest).toHaveBeenCalledWith(expect.anything(), "state-1");
  });

  it("completes a redeemed OIDC login and deletes the transient state", async () => {
    dbMocks.getOidcAuthRequest.mockResolvedValue(
      authRequest({ session_token: "session-1" }),
    );
    dbMocks.getAuthSession.mockResolvedValue(sessionRecord());

    const response = await runWorker(
      jsonRequest("/v1/auth/oidc/complete", { state: "state-1" }),
      makeEnv(),
    );

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toMatchObject({
      session_token: "session-1",
      asn: "4242421024",
      effective_mnt: "IRIS-MNT",
      auth_method: {
        kind: "oidc",
        provider: "kioubit",
      },
    });
    expect(dbMocks.deleteOidcAuthRequest).toHaveBeenCalledWith(expect.anything(), "state-1");
  });
});
