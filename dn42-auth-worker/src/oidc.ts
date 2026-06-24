import { createRemoteJWKSet, jwtVerify } from "jose";

import { SESSION_TTL_SECONDS } from "./auth";
import type {
  AuthMethod,
  MaintainerRecord,
  OidcAuthRequestRecord,
  OidcClaimPath,
  OidcProviderConfig,
  OidcProviderDiscovery,
  OidcTokenEndpointAuthMethod,
  OidcTokenResponse,
  SessionRecord,
  UiMessage,
} from "./types";
import { HttpError, addSeconds, nowIso, randomBase64Url, readNamedSecret, uiMessage } from "./utils";

const OIDC_AUTH_TTL_SECONDS = 15 * 60;
const DEFAULT_OIDC_SCOPES = ["openid", "profile", "email"];

type JsonObject = Record<string, unknown>;

export function oidcMethodsFromProviders(providers: OidcProviderConfig[]): AuthMethod[] {
  return providers.map((provider) => ({
    kind: "oidc",
    provider: provider.name,
    label: uiMessage(provider.label),
    description: provider.description
      ? uiMessage(provider.description)
      : uiMessage("auth_method.oidc.description", { provider: provider.label }),
  }));
}

export function rewriteIssuerHost(url: string, provider: OidcProviderConfig): string {
  if (!provider.dn42_issuer) return url;
  const original = new URL(url);
  const issuer = new URL(provider.issuer);
  if (original.host !== issuer.host) return url;
  const dn42 = new URL(provider.dn42_issuer);
  original.protocol = dn42.protocol;
  original.host = dn42.host;
  return original.toString();
}

export function oidcProviderByName(
  providers: OidcProviderConfig[],
  name: string,
): OidcProviderConfig | undefined {
  return providers.find((provider) => provider.name === name);
}

async function sha256Base64Url(input: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(input));
  const bytes = new Uint8Array(digest);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function jsonObject(value: unknown, message: UiMessage): JsonObject {
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    return value as JsonObject;
  }
  throw new HttpError(message, 502);
}

function claimPathSegments(path: string): string[] {
  return path
    .split(".")
    .map((segment) => segment.trim())
    .filter(Boolean);
}

function normalizedClaimPaths(pathOrPaths: OidcClaimPath): string[] {
  const rawPaths = Array.isArray(pathOrPaths) ? pathOrPaths : [pathOrPaths];
  return rawPaths.map((path) => path.trim()).filter(Boolean);
}

function describeClaimPaths(pathOrPaths: OidcClaimPath): string {
  return normalizedClaimPaths(pathOrPaths).join(" or ");
}

export function claimValueAtPath(source: unknown, path: string): unknown {
  let current: unknown = source;
  for (const segment of claimPathSegments(path)) {
    if (typeof current !== "object" || current === null || Array.isArray(current)) {
      return undefined;
    }
    current = (current as JsonObject)[segment];
  }
  return current;
}

function firstClaimValue(sources: JsonObject[], pathOrPaths: OidcClaimPath): unknown {
  const paths = normalizedClaimPaths(pathOrPaths);
  for (const source of sources) {
    for (const path of paths) {
      const value = claimValueAtPath(source, path);
      if (value !== undefined && value !== null) {
        return value;
      }
    }
  }
  return undefined;
}

function claimValueToString(value: unknown): string | undefined {
  if (typeof value === "string" && value.trim().length > 0) {
    return value.trim();
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return String(value);
  }
  return undefined;
}

function claimValueToStrings(value: unknown): string[] {
  const direct = claimValueToString(value);
  if (direct) {
    return [direct];
  }

  if (!Array.isArray(value)) {
    return [];
  }

  return value
    .map((entry) => claimValueToString(entry))
    .filter((entry): entry is string => Boolean(entry));
}

function resolveAsnClaim(sources: JsonObject[], provider: OidcProviderConfig): string {
  const value = firstClaimValue(sources, provider.asn_claim);
  const asn = claimValueToString(value)?.replace(/^AS/i, "");
  if (!asn) {
    throw new HttpError(
      uiMessage("error.oidc.claim.asn_missing", {
        claim: describeClaimPaths(provider.asn_claim),
      }),
      400,
    );
  }
  return asn;
}

function resolveMaintainerClaim(
  sources: JsonObject[],
  provider: OidcProviderConfig,
  maintainers: MaintainerRecord[],
): string {
  const rawClaim = firstClaimValue(sources, provider.mntner_claim);
  const claimCandidates = claimValueToStrings(rawClaim).map((value) => value.toUpperCase());
  if (claimCandidates.length === 0) {
    throw new HttpError(
      uiMessage("error.oidc.claim.maintainer_missing", {
        claim: describeClaimPaths(provider.mntner_claim),
      }),
      400,
    );
  }

  const maintainerNames = new Map(
    maintainers.map((maintainer) => [maintainer.name.toUpperCase(), maintainer.name]),
  );
  for (const candidate of claimCandidates) {
    const matched = maintainerNames.get(candidate);
    if (matched) {
      return matched;
    }
  }

  throw new HttpError(
    uiMessage("error.oidc.maintainer.not_in_mnt_by", {
      provider: provider.label,
      candidates: claimCandidates.join(", "),
    }),
    400,
  );
}

function buildOidcSession(
  provider: OidcProviderConfig,
  asn: string,
  effectiveMnt: string,
): SessionRecord {
  const createdAt = nowIso();
  return {
    token: crypto.randomUUID(),
    asn,
    effective_mnt: effectiveMnt,
    auth_method: {
      kind: "oidc",
      provider: provider.name,
      label: uiMessage(provider.label),
      description: uiMessage("auth_method.oidc.session_description", {
        provider: provider.label,
        mnt: effectiveMnt,
      }),
    },
    created_at: createdAt,
    expires_at: addSeconds(createdAt, SESSION_TTL_SECONDS),
  };
}

export function discoveryUrlForProvider(provider: OidcProviderConfig): URL {
  if (provider.discovery_url) {
    return new URL(provider.discovery_url);
  }

  const issuer = new URL(provider.issuer);
  const discovery = new URL(issuer);
  const issuerPath = issuer.pathname.replace(/^\/+|\/+$/g, "");
  discovery.pathname = `/.well-known/openid-configuration${issuerPath ? `/${issuerPath}` : ""}`;
  discovery.search = "";
  discovery.hash = "";
  return discovery;
}

export function jwksUrlForProvider(
  provider: OidcProviderConfig,
  discovery?: Pick<OidcProviderDiscovery, "jwks_uri">,
): URL {
  if (provider.jwks_uri) {
    return new URL(provider.jwks_uri);
  }
  if (discovery?.jwks_uri) {
    return new URL(discovery.jwks_uri);
  }

  const issuer = new URL(provider.issuer);
  const jwks = new URL(issuer);
  const issuerPath = jwks.pathname.endsWith("/") ? jwks.pathname : `${jwks.pathname}/`;
  jwks.pathname = `${issuerPath}.well-known/jwks.json`;
  jwks.search = "";
  jwks.hash = "";
  return jwks;
}

export async function fetchOidcDiscovery(provider: OidcProviderConfig): Promise<OidcProviderDiscovery> {
  if (provider.authorization_endpoint && provider.token_endpoint && provider.jwks_uri) {
    return {
      issuer: provider.issuer,
      authorization_endpoint: provider.authorization_endpoint,
      token_endpoint: provider.token_endpoint,
      jwks_uri: provider.jwks_uri,
      userinfo_endpoint: provider.userinfo_endpoint,
    };
  }

  const response = await fetch(discoveryUrlForProvider(provider));
  if (!response.ok) {
    throw new HttpError(
      uiMessage("error.oidc.discovery.failed", {
        provider: provider.label,
        status: String(response.status),
      }),
      502,
    );
  }

  const body = jsonObject(
    await response.json().catch(() => null),
    uiMessage("error.oidc.discovery.invalid_json", { provider: provider.label }),
  );

  return {
    issuer: claimValueToString(body.issuer) ?? provider.issuer,
    authorization_endpoint:
      provider.authorization_endpoint ??
      claimValueToString(body.authorization_endpoint) ??
      (() => {
        throw new HttpError(
          uiMessage("error.oidc.discovery.missing_field", {
            provider: provider.label,
            field: "authorization_endpoint",
          }),
          502,
        );
      })(),
    token_endpoint:
      provider.token_endpoint ??
      claimValueToString(body.token_endpoint) ??
      (() => {
        throw new HttpError(
          uiMessage("error.oidc.discovery.missing_field", {
            provider: provider.label,
            field: "token_endpoint",
          }),
          502,
        );
      })(),
    jwks_uri:
      provider.jwks_uri ??
      claimValueToString(body.jwks_uri) ??
      (() => {
        throw new HttpError(
          uiMessage("error.oidc.discovery.missing_field", {
            provider: provider.label,
            field: "jwks_uri",
          }),
          502,
        );
      })(),
    userinfo_endpoint:
      provider.userinfo_endpoint ?? claimValueToString(body.userinfo_endpoint),
  };
}

function providerScopes(provider: OidcProviderConfig): string {
  return (provider.scopes?.length ? provider.scopes : DEFAULT_OIDC_SCOPES).join(" ");
}

export async function createOidcAuthorizationRequest(
  provider: OidcProviderConfig,
  discovery: OidcProviderDiscovery,
  challengeId: string,
  redirectUri: string,
): Promise<{ authorizationUrl: string; record: OidcAuthRequestRecord }> {
  const state = randomBase64Url(24);
  const nonce = randomBase64Url(24);
  const codeVerifier = randomBase64Url(48);
  const codeChallenge = await sha256Base64Url(codeVerifier);
  const createdAt = nowIso();
  const authorizationUrl = new URL(discovery.authorization_endpoint);
  authorizationUrl.searchParams.set("response_type", "code");
  authorizationUrl.searchParams.set("client_id", provider.client_id);
  authorizationUrl.searchParams.set("redirect_uri", redirectUri);
  authorizationUrl.searchParams.set("scope", providerScopes(provider));
  authorizationUrl.searchParams.set("state", state);
  authorizationUrl.searchParams.set("nonce", nonce);
  authorizationUrl.searchParams.set("code_challenge", codeChallenge);
  authorizationUrl.searchParams.set("code_challenge_method", "S256");

  return {
    authorizationUrl: authorizationUrl.toString(),
    record: {
      state,
      challenge_id: challengeId,
      provider: provider.name,
      nonce,
      code_verifier: codeVerifier,
      redirect_uri: redirectUri,
      session_token: null,
      created_at: createdAt,
      expires_at: addSeconds(createdAt, OIDC_AUTH_TTL_SECONDS),
    },
  };
}

function tokenEndpointAuthMethod(provider: OidcProviderConfig): OidcTokenEndpointAuthMethod {
  return provider.token_endpoint_auth_method ?? "client_secret_post";
}

export async function exchangeAuthorizationCode(
  env: object,
  provider: OidcProviderConfig,
  discovery: OidcProviderDiscovery,
  code: string,
  redirectUri: string,
  codeVerifier: string,
): Promise<OidcTokenResponse> {
  const params = new URLSearchParams();
  params.set("grant_type", "authorization_code");
  params.set("code", code);
  params.set("client_id", provider.client_id);
  params.set("redirect_uri", redirectUri);
  params.set("code_verifier", codeVerifier);

  const headers = new Headers({
    "content-type": "application/x-www-form-urlencoded",
  });

  switch (tokenEndpointAuthMethod(provider)) {
    case "client_secret_post": {
      if (!provider.client_secret_env) {
        throw new HttpError(
          uiMessage("error.oidc.client_secret.missing", {
            provider: provider.label,
            method: "client_secret_post",
          }),
          500,
        );
      }
      params.set("client_secret", readNamedSecret(env, provider.client_secret_env));
      break;
    }
    case "client_secret_basic": {
      if (!provider.client_secret_env) {
        throw new HttpError(
          uiMessage("error.oidc.client_secret.missing", {
            provider: provider.label,
            method: "client_secret_basic",
          }),
          500,
        );
      }
      const secret = readNamedSecret(env, provider.client_secret_env);
      headers.set("authorization", `Basic ${btoa(`${provider.client_id}:${secret}`)}`);
      break;
    }
    case "none":
      break;
  }

  const response = await fetch(discovery.token_endpoint, {
    method: "POST",
    headers,
    body: params.toString(),
  });
  const body = jsonObject(
    await response.json().catch(() => null),
    uiMessage("error.oidc.token.invalid_json", { provider: provider.label }),
  );

  if (!response.ok) {
    const description =
      claimValueToString(body.error_description) ??
      claimValueToString(body.error) ??
      `HTTP ${response.status}`;
    throw new HttpError(
      uiMessage("error.oidc.token.rejected", {
        provider: provider.label,
        description,
      }),
      400,
    );
  }

  return body as OidcTokenResponse;
}

async function fetchUserInfo(
  discovery: OidcProviderDiscovery,
  provider: OidcProviderConfig,
  accessToken: string,
): Promise<JsonObject | null> {
  if (!discovery.userinfo_endpoint) {
    return null;
  }

  const response = await fetch(discovery.userinfo_endpoint, {
    headers: {
      authorization: `Bearer ${accessToken}`,
    },
  });
  if (!response.ok) {
    throw new HttpError(
      uiMessage("error.oidc.userinfo.failed", {
        provider: provider.label,
        status: String(response.status),
      }),
      502,
    );
  }

  return jsonObject(
    await response.json().catch(() => null),
    uiMessage("error.oidc.userinfo.invalid_json", { provider: provider.label }),
  );
}

export async function verifiedOidcClaimSources(
  tokenResponse: OidcTokenResponse,
  provider: OidcProviderConfig,
  discovery: OidcProviderDiscovery,
  expectedNonce: string,
): Promise<JsonObject[]> {
  if (!tokenResponse.id_token) {
    throw new HttpError(
      uiMessage("error.oidc.id_token.missing", { provider: provider.label }),
      400,
    );
  }

  const jwks = createRemoteJWKSet(jwksUrlForProvider(provider, discovery));
  const verified = await jwtVerify(tokenResponse.id_token, jwks, {
    issuer: provider.issuer,
    audience: provider.audience,
  }).catch((error) => {
    throw new HttpError(
      uiMessage("error.oidc.id_token.invalid", {
        provider: provider.label,
        detail: error instanceof Error ? error.message : "unknown error",
      }),
      400,
    );
  });

  const payload = verified.payload as JsonObject;
  const nonce = claimValueToString(payload.nonce);
  if (nonce !== expectedNonce) {
    throw new HttpError(
      uiMessage("error.oidc.id_token.invalid_nonce", { provider: provider.label }),
      400,
    );
  }

  const claimSources = [payload];
  const needsUserInfo =
    firstClaimValue(claimSources, provider.asn_claim) === undefined ||
    firstClaimValue(claimSources, provider.mntner_claim) === undefined;

  if (needsUserInfo && tokenResponse.access_token) {
    const userInfo = await fetchUserInfo(discovery, provider, tokenResponse.access_token);
    if (userInfo) {
      claimSources.push(userInfo);
    }
  }

  return claimSources;
}

export function oidcAsnFromClaimSources(
  claimSources: JsonObject[],
  provider: OidcProviderConfig,
): string {
  return resolveAsnClaim(claimSources, provider);
}

export function oidcMaintainerFromClaimSources(
  claimSources: JsonObject[],
  provider: OidcProviderConfig,
  maintainers: MaintainerRecord[],
): string {
  return resolveMaintainerClaim(claimSources, provider, maintainers);
}

export async function verifyOidcToken(
  tokenResponse: OidcTokenResponse,
  provider: OidcProviderConfig,
  discovery: OidcProviderDiscovery,
  expectedNonce: string,
  asn: string,
  maintainers: MaintainerRecord[],
): Promise<SessionRecord> {
  const claimSources = await verifiedOidcClaimSources(
    tokenResponse,
    provider,
    discovery,
    expectedNonce,
  );
  const tokenAsn = resolveAsnClaim(claimSources, provider);
  if (tokenAsn !== asn) {
    throw new HttpError(
      uiMessage("error.oidc.asn.mismatch", {
        token_asn: tokenAsn,
        requested_asn: asn,
      }),
      400,
    );
  }

  const effectiveMnt = resolveMaintainerClaim(claimSources, provider, maintainers);
  return buildOidcSession(provider, asn, effectiveMnt);
}

export function sessionFromOidcIdentity(
  provider: OidcProviderConfig,
  asn: string,
  effectiveMnt: string,
): SessionRecord {
  return buildOidcSession(provider, asn, effectiveMnt);
}
