import {
  SESSION_TTL_SECONDS,
  assertChallengeFresh,
  createChallenge,
  createRegistryEmailAuthRequest,
  createRegistryEmailSession,
  lookupPgpKeyOnKeyservers,
  normalizePgpFingerprint,
  verifyRegistryPgpChallenge,
  verifyRegistrySshChallenge,
} from "dn42-auth-worker/auth";
import {
  consumeFreshChallenge,
  consumeCompletedRegistryEmailAuthRequestByToken,
  deleteChallenge,
  deleteRegistryEmailAuthRequest,
  getAuthSession,
  getChallenge,
  getOidcAuthRequest,
  getRegistryEmailAuthRequest,
  getRegistryEmailAuthRequestByToken,
  deleteOidcAuthRequest,
  putAuthSession,
  putChallenge,
  putOidcAuthRequest,
  putRegistryEmailAuthRequest,
} from "dn42-auth-worker/db";
import {
  createOidcAuthorizationRequest,
  exchangeAuthorizationCode,
  fetchOidcDiscovery,
  oidcAsnFromClaimSources,
  oidcMaintainerFromClaimSources,
  oidcMethodsFromProviders,
  oidcProviderByName,
  sessionFromOidcIdentity,
  verifiedOidcClaimSources,
} from "dn42-auth-worker/oidc";
import {
  NoMaintainerError,
  RegistryPathNotFoundError,
  loadMaintainersForAsn,
  methodsFromMaintainers,
} from "dn42-auth-worker/registry";
import { sendRegistryEmailAuthMessage } from "dn42-auth-worker/mailer";
import { resolveLocaleCode, translator } from "./i18n";
import {
  AuthStartSchema,
  HostImpersonationSchema,
  RegistrySshVerifySchema,
  RegistryPgpVerifySchema,
  RegistryEmailSendSchema,
  RegistryEmailVerifySchema,
  RegistryEmailCompleteSchema,
  OidcStartSchema,
  OidcCompleteSchema,
} from "dn42-auth-worker/schemas";
import type {
  AuthStartRequest,
  HostImpersonationRequest,
  RegistrySshVerifyRequest,
  RegistryPgpVerifyRequest,
  RegistryEmailSendRequest,
  RegistryEmailVerifyRequest,
  RegistryEmailCompleteRequest,
  OidcStartRequest,
  OidcCompleteRequest,
} from "dn42-auth-worker/schemas";
import type {
  AuthSessionResponse,
  AuthStartResponse,
  ChallengeRecord,
  MaintainerRecord,
  OidcProviderConfig,
  OidcStartResponse,
  PgpKeyLookupResponse,
  RegistryEmailSendResponse,
  RegistryEmailTarget,
  SessionRecord,
  UiMessage,
} from "dn42-auth-worker/types";
import type { ZodType, core } from "zod";
import {
  HttpError,
  I18nError,
  addSeconds,
  bearerToken,
  isExpired,
  isUiMessageKey,
  normalizeSupportedAutopeerAsn,
  nowIso,
  parseConfiguredAsns,
  readOptionalEnvString,
  timingSafeEqual,
  toUiMessage,
  uiMessage,
} from "dn42-auth-worker/utils";
import { OPENAPI_PATH, SWAGGER_PATH, openApiSpec, swaggerUiHtml } from "./docs";

interface Env {
  DB: D1Database;
  DN42_REGISTRY_OWNER: string;
  DN42_REGISTRY_REPO: string;
  DN42_REGISTRY_BRANCH: string;
  DN42_REGISTRY_BASE_URL: string;
  HOST_ASNS: string;
  AUTH_SITE_URL: string;
  ALLOWED_RETURN_URLS: string;
  OIDC_PROVIDERS: string;
  [key: string]: unknown;
}

interface ApiError {
  error: UiMessage;
}

const OIDC_CALLBACK_PREFIX = "/oidc/callback/";
const REGISTRY_EMAIL_CALLBACK_PATH = "/auth/email/callback";

const SECURITY_HEADERS: Record<string, string> = {
  "x-content-type-options": "nosniff",
  "x-frame-options": "DENY",
  "referrer-policy": "strict-origin-when-cross-origin",
};

function buildCorsHeaders(request: Request, env: Env): Headers {
  const headers = new Headers();
  headers.set("access-control-allow-methods", "GET,POST,OPTIONS");
  headers.set("access-control-allow-headers", "authorization,content-type,x-return-to");
  headers.set("access-control-max-age", "86400");
  const origin = request.headers.get("origin");
  if (origin && isAllowedReturnUrl(env, origin)) {
    headers.set("access-control-allow-origin", origin);
    headers.set("vary", "origin");
  }
  return headers;
}

function jsonWithCors(request: Request, env: Env, body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body, null, 2), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      ...SECURITY_HEADERS,
      ...Object.fromEntries(buildCorsHeaders(request, env)),
    },
  });
}

function errorWithCors(request: Request, env: Env, message: string | UiMessage, status = 400): Response {
  const body: ApiError = { error: toUiMessage(message) };
  return jsonWithCors(request, env, body, status);
}

async function readJson<T>(request: Request): Promise<T> {
  try {
    return (await request.json()) as T;
  } catch {
    throw new HttpError("error.request.body.invalid_json", 400);
  }
}

function issueToHttpError(issue: core.$ZodIssue, rootLabel: string): HttpError {
  const field = issue.path.length === 0 ? rootLabel : issue.path.map(String).join(".");
  if (issue.code === "custom") {
    const params = (issue.params ?? {}) as { uiKey?: unknown; literal?: unknown };
    if (typeof params.uiKey === "string") {
      if (params.literal === true) return new HttpError(params.uiKey, 400);
      return new HttpError(uiMessage(params.uiKey, { field }), 400);
    }
  }
  if (issue.code === "invalid_type") {
    switch (issue.expected) {
      case "string": return new HttpError(uiMessage("error.field.must_be_string", { field }), 400);
      case "boolean": return new HttpError(uiMessage("error.field.must_be_boolean", { field }), 400);
      case "int": case "number": return new HttpError(uiMessage("error.field.must_be_integer", { field }), 400);
    }
  }
  return new HttpError(uiMessage("error.field.required", { field }), 400);
}

async function parseBody<T>(request: Request, schema: ZodType<T, unknown>, rootLabel = "request body"): Promise<T> {
  const raw = await readJson<unknown>(request);
  const result = schema.safeParse(raw);
  if (result.success) return result.data;
  throw issueToHttpError(result.error.issues[0]!, rootLabel);
}

function parseJsonEnv<T>(raw: string, field: string): T {
  try {
    return JSON.parse(raw) as T;
  } catch (error) {
    throw new Error(`${field} must be valid JSON: ${String(error)}`);
  }
}

function configuredOidcProviders(env: Env): OidcProviderConfig[] {
  return parseJsonEnv(env.OIDC_PROVIDERS, "OIDC_PROVIDERS");
}

function registryEmailAuthConfigured(env: Env): boolean {
  return readOptionalEnvString(env, "RESEND_API_KEY") !== null;
}

function allowedReturnUrls(env: Env): Set<string> {
  const urls = new Set<string>();
  urls.add(env.AUTH_SITE_URL);
  for (const entry of (env.ALLOWED_RETURN_URLS ?? "").split(",")) {
    const trimmed = entry.trim();
    if (trimmed) urls.add(trimmed);
  }
  return urls;
}

function isAllowedReturnUrl(env: Env, origin: string): boolean {
  return allowedReturnUrls(env).has(origin);
}

function siteReturnUrlFromRequest(request: Request, env: Env): string {
  for (const name of ["X-Return-To", "Origin", "Referer"]) {
    const value = request.headers.get(name)?.trim();
    if (!value) continue;
    try {
      const url = new URL(value);
      if ((url.protocol === "https:" || url.protocol === "http:") && isAllowedReturnUrl(env, url.origin)) {
        return url.origin;
      }
    } catch { /* fall through */ }
  }
  return env.AUTH_SITE_URL;
}

function siteRedirectResponse(siteReturnUrl: string, fragment: string): Response {
  const target = new URL(siteReturnUrl);
  target.hash = fragment;
  return Response.redirect(target.toString(), 302);
}

function sessionRedirectFragment(env: Env, session: SessionRecord, lang?: string | null): string {
  const resp = authSessionResponseForEnv(env, session);
  let frag = `auth_session=${encodeURIComponent(JSON.stringify(resp))}`;
  if (lang) frag += `&lang=${encodeURIComponent(lang)}`;
  return frag;
}

function fragmentMessage(name: string, message: string | UiMessage): string {
  return `${name}=${encodeURIComponent(JSON.stringify(toUiMessage(message)))}`;
}

function returnToFromQuery(url: URL, env: Env): string {
  const raw = url.searchParams.get("return_to")?.trim();
  if (raw && isAllowedReturnUrl(env, raw)) return raw;
  return env.AUTH_SITE_URL;
}

function returnToFromRequestBody(returnTo: string | null | undefined, env: Env): string | null {
  const raw = returnTo?.trim();
  if (!raw) return null;
  if (isAllowedReturnUrl(env, raw)) return raw;
  throw new HttpError("error.auth.return_to.invalid", 400);
}

function langFromQuery(url: URL): string | null {
  return url.searchParams.get("lang")?.trim() || null;
}

function oidcCallbackUrl(env: Env, providerName: string): string {
  const base = env.AUTH_SITE_URL.endsWith("/") ? env.AUTH_SITE_URL : `${env.AUTH_SITE_URL}/`;
  return `${base}oidc/callback/${encodeURIComponent(providerName)}`;
}

function registryEmailCallbackUrl(env: Env, challengeId: string, token: string): string {
  const base = env.AUTH_SITE_URL.endsWith("/") ? env.AUTH_SITE_URL : `${env.AUTH_SITE_URL}/`;
  const callback = new URL(REGISTRY_EMAIL_CALLBACK_PATH, base);
  callback.searchParams.set("challenge_id", challengeId);
  callback.searchParams.set("token", token);
  return callback.toString();
}

function sessionCanImpersonate(env: Env, session: SessionRecord): boolean {
  return parseConfiguredAsns(env.HOST_ASNS).has(session.asn);
}

function authSessionResponseForEnv(env: Env, session: SessionRecord): AuthSessionResponse {
  return {
    session_token: session.token,
    asn: session.asn,
    effective_mnt: session.effective_mnt,
    auth_method: session.auth_method,
    can_impersonate: sessionCanImpersonate(env, session),
    expires_at: session.expires_at,
  };
}

async function requireSession(env: Env, request: Request): Promise<SessionRecord> {
  const token = bearerToken(request);
  if (!token) throw new HttpError("error.auth.session.token.missing", 401);
  const session = await getAuthSession(env, token);
  if (!session) throw new HttpError("error.auth.session.unknown", 401);
  if (isExpired(session.expires_at)) throw new HttpError("error.auth.session.expired", 401);
  return session;
}

function classifyMaintainerLookupError(asn: string, error: unknown): HttpError {
  if (error instanceof RegistryPathNotFoundError) return new HttpError(uiMessage("error.auth.asn.not_found", { asn }), 400);
  if (error instanceof NoMaintainerError) return new HttpError(uiMessage("error.auth.asn.no_supported_auth", { asn }), 400);
  if (error instanceof HttpError) return error;
  return new HttpError(uiMessage("error.registry.lookup_failed", { asn }), 502);
}

async function loadMaintainersForRequestAsn(env: Env, asn: string): Promise<MaintainerRecord[]> {
  try {
    return await loadMaintainersForAsn(env, asn);
  } catch (error) {
    throw classifyMaintainerLookupError(asn, error);
  }
}

function resolveEffectiveMaintainer(maintainers: MaintainerRecord[], requestedMaintainer?: string | null): string {
  if (maintainers.length === 0) throw new HttpError("error.auth.impersonation.no_maintainers", 400);
  const available = [...new Set(maintainers.map((m) => m.name))].join(", ");
  if (requestedMaintainer) {
    const requested = requestedMaintainer.trim().toUpperCase();
    const matched = maintainers.find((m) => m.name.toUpperCase() === requested);
    if (!matched) throw new HttpError(uiMessage("error.auth.impersonation.maintainer.missing", { requested, available }), 400);
    return matched.name;
  }
  if (maintainers.length === 1) return maintainers[0]!.name;
  throw new HttpError(uiMessage("error.auth.impersonation.maintainer.required", { available }), 400);
}

async function consumeChallengeOrThrow(env: Env, challengeId: string): Promise<ChallengeRecord> {
  const result = await consumeFreshChallenge(env, challengeId);
  switch (result.kind) {
    case "available": return result.challenge;
    case "missing": throw new HttpError("error.auth.challenge.unknown_id", 404);
    case "expired": throw new HttpError("error.auth.challenge.expired", 400);
    case "consumed": throw new HttpError("error.auth.challenge.used", 400);
  }
}

function registryEmailTargetsForChallenge(challenge: ChallengeRecord): RegistryEmailTarget[] {
  const methodTargets = challenge.methods.find((m) => m.kind === "registry_email")?.email_targets ?? [];
  if (methodTargets.length > 0) return methodTargets.filter((t) => t.emails.length > 0);
  return challenge.maintainers
    .map((m) => ({ maintainer: m.name, emails: m.contact_emails ?? [] }))
    .filter((t) => t.emails.length > 0);
}

function resolveRegistryEmailTarget(challenge: ChallengeRecord, requestedMaintainer?: string | null): RegistryEmailTarget {
  const targets = registryEmailTargetsForChallenge(challenge);
  if (targets.length === 0) throw new HttpError(uiMessage("error.auth.registry_email.contacts.missing", { asn: challenge.asn }), 400);
  if (requestedMaintainer) {
    const requested = requestedMaintainer.trim().toUpperCase();
    const matched = targets.find((t) => t.maintainer.toUpperCase() === requested);
    if (!matched) throw new HttpError(uiMessage("error.auth.registry_email.target.missing", { requested }), 400);
    return matched;
  }
  if (targets.length === 1) return targets[0]!;
  throw new HttpError(uiMessage("error.auth.registry_email.target.required"), 400);
}

async function createCompletedRegistryEmailSession(env: Env, challengeId: string, effectiveMnt: string): Promise<SessionRecord> {
  const challenge = await consumeChallengeOrThrow(env, challengeId);
  const session = createRegistryEmailSession(challenge, effectiveMnt);
  await putAuthSession(env, session);
  return session;
}

function errorMessage(error: unknown, fallbackKey: string): UiMessage {
  if (error instanceof HttpError) return error.uiMessage;
  if (error instanceof I18nError) return error.uiMessage;
  return uiMessage(fallbackKey);
}

async function router(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const method = request.method;

  if (method === "OPTIONS") {
    return new Response(null, { status: 204, headers: buildCorsHeaders(request, env) });
  }

  if (method === "GET" && url.pathname === "/health") {
    return jsonWithCors(request, env, { ok: true, now: nowIso() });
  }

  if (method === "GET" && url.pathname === OPENAPI_PATH) {
    return jsonWithCors(request, env, openApiSpec(request, env.AUTH_SITE_URL));
  }

  if (
    method === "GET" &&
    (url.pathname === SWAGGER_PATH || url.pathname === `${SWAGGER_PATH}/`)
  ) {
    return new Response(swaggerUiHtml(OPENAPI_PATH), {
      headers: {
        "content-type": "text/html; charset=utf-8",
        ...SECURITY_HEADERS,
      },
    });
  }

  if (method === "GET" && url.pathname === "/config.json") {
    return jsonWithCors(request, env, {
      autopeer_api_url: env.AUTH_SITE_URL,
      oidc_methods: oidcMethodsFromProviders(configuredOidcProviders(env)),
      allowed_return_urls: Array.from(allowedReturnUrls(env)),
    });
  }

  // POST /v1/auth/start
  if (method === "POST" && url.pathname === "/v1/auth/start") {
    const body = await parseBody<AuthStartRequest>(request, AuthStartSchema);
    const asn = normalizeSupportedAutopeerAsn(body.asn);
    const maintainers = await loadMaintainersForRequestAsn(env, asn);
    const challenge = createChallenge(asn);
    challenge.maintainers = maintainers;
    challenge.methods = methodsFromMaintainers(maintainers, [], {
      registryEmailEnabled: registryEmailAuthConfigured(env),
    });
    if (challenge.methods.length === 0) {
      const key = configuredOidcProviders(env).length > 0
        ? "error.auth.asn.no_registry_auth.oidc_hint"
        : "error.auth.asn.no_supported_auth";
      throw new HttpError(uiMessage(key, { asn }), 400);
    }
    await putChallenge(env, challenge);
    const response: AuthStartResponse = {
      asn,
      challenge_id: challenge.id,
      challenge_text: challenge.challenge_text,
      challenge_ttl_seconds: 15 * 60,
      methods: challenge.methods,
    };
    return jsonWithCors(request, env, response);
  }

  // POST /v1/auth/impersonate
  if (method === "POST" && url.pathname === "/v1/auth/impersonate") {
    const impersonatorSession = await requireSession(env, request);
    if (!sessionCanImpersonate(env, impersonatorSession)) {
      throw new HttpError(uiMessage("error.auth.impersonation.asn.not_host", { asn: impersonatorSession.asn }), 403);
    }
    const body = await parseBody<HostImpersonationRequest>(request, HostImpersonationSchema);
    const asn = normalizeSupportedAutopeerAsn(body.asn);
    const maintainers = await loadMaintainersForRequestAsn(env, asn);
    const effectiveMnt = resolveEffectiveMaintainer(maintainers, body.effective_mnt);
    const createdAt = nowIso();
    const session: SessionRecord = {
      token: crypto.randomUUID(),
      asn,
      effective_mnt: effectiveMnt,
      auth_method: {
        kind: "host_impersonation",
        label: uiMessage("auth_method.host_impersonation.label"),
        description: uiMessage("auth_method.host_impersonation.description", { mnt: effectiveMnt, host_asn: impersonatorSession.asn }),
        provider: `AS${impersonatorSession.asn}`,
      },
      created_at: createdAt,
      expires_at: addSeconds(createdAt, SESSION_TTL_SECONDS),
    };
    await putAuthSession(env, session);
    return jsonWithCors(request, env, authSessionResponseForEnv(env, session));
  }

  // POST /v1/auth/verify/registry-ssh
  if (method === "POST" && url.pathname === "/v1/auth/verify/registry-ssh") {
    const body = await parseBody<RegistrySshVerifyRequest>(request, RegistrySshVerifySchema);
    const challenge = await consumeChallengeOrThrow(env, body.challenge_id);
    const session = await verifyRegistrySshChallenge(challenge, body);
    await putAuthSession(env, session);
    return jsonWithCors(request, env, authSessionResponseForEnv(env, session));
  }

  // POST /v1/auth/verify/registry-pgp
  if (method === "POST" && url.pathname === "/v1/auth/verify/registry-pgp") {
    const body = await parseBody<RegistryPgpVerifyRequest>(request, RegistryPgpVerifySchema);
    const challenge = await consumeChallengeOrThrow(env, body.challenge_id);
    const session = await verifyRegistryPgpChallenge(challenge, body);
    await putAuthSession(env, session);
    return jsonWithCors(request, env, authSessionResponseForEnv(env, session));
  }

  // GET /v1/auth/lookup/pgp-key
  if (method === "GET" && url.pathname === "/v1/auth/lookup/pgp-key") {
    const rawFingerprint = url.searchParams.get("fingerprint") ?? "";
    const normalized = normalizePgpFingerprint(rawFingerprint);
    if (!normalized) throw new HttpError("error.auth.pgp.invalid_fingerprint", 400);
    const result = await lookupPgpKeyOnKeyservers(normalized);
    const response: PgpKeyLookupResponse = result.publicKey
      ? { fingerprint: result.fingerprint, found: true, public_key: result.publicKey, source: result.source ?? undefined }
      : { fingerprint: result.fingerprint, found: false };
    return jsonWithCors(request, env, response);
  }

  // POST /v1/auth/verify/registry-email/send
  if (method === "POST" && url.pathname === "/v1/auth/verify/registry-email/send") {
    if (!registryEmailAuthConfigured(env)) throw new HttpError("error.auth.registry_email.unavailable", 503);
    const body = await parseBody<RegistryEmailSendRequest>(request, RegistryEmailSendSchema);
    const challenge = await getChallenge(env, body.challenge_id);
    if (!challenge) throw new HttpError("error.auth.challenge.unknown_id", 404);
    assertChallengeFresh(challenge);
    const target = resolveRegistryEmailTarget(challenge, body.effective_mnt);
    const locale = body.locale ?? "en";
    const emailAuthRequest = createRegistryEmailAuthRequest(challenge, target.maintainer, target.emails, locale);
    const siteReturnUrl = siteReturnUrlFromRequest(request, env);
    await sendRegistryEmailAuthMessage(
      env,
      translator(resolveLocaleCode(locale) ?? "en"),
      challenge.asn,
      target.maintainer,
      emailAuthRequest,
      registryEmailCallbackUrl(env, challenge.id, emailAuthRequest.token),
    );
    await putRegistryEmailAuthRequest(env, { ...emailAuthRequest, site_return_url: siteReturnUrl });
    const response: RegistryEmailSendResponse = {
      effective_mnt: target.maintainer,
      emails: target.emails,
      expires_at: emailAuthRequest.expires_at,
    };
    return jsonWithCors(request, env, response);
  }

  // POST /v1/auth/verify/registry-email
  if (method === "POST" && url.pathname === "/v1/auth/verify/registry-email") {
    const body = await parseBody<RegistryEmailVerifyRequest>(request, RegistryEmailVerifySchema);
    const emailAuthRequest = await getRegistryEmailAuthRequest(env, body.challenge_id);
    if (!emailAuthRequest) throw new HttpError("error.auth.registry_email.state.missing", 404);
    if (emailAuthRequest.session_token) throw new HttpError("error.auth.registry_email.already_completed", 409);
    if (isExpired(emailAuthRequest.expires_at)) {
      await deleteRegistryEmailAuthRequest(env, body.challenge_id);
      throw new HttpError("error.auth.registry_email.state.expired", 400);
    }
    if (!timingSafeEqual(body.code, emailAuthRequest.code)) throw new HttpError("error.auth.registry_email.code.invalid", 400);
    const session = await createCompletedRegistryEmailSession(env, body.challenge_id, emailAuthRequest.effective_mnt);
    await deleteRegistryEmailAuthRequest(env, body.challenge_id);
    return jsonWithCors(request, env, authSessionResponseForEnv(env, session));
  }

  // POST /v1/auth/verify/registry-email/complete
  if (method === "POST" && url.pathname === "/v1/auth/verify/registry-email/complete") {
    const body = await parseBody<RegistryEmailCompleteRequest>(request, RegistryEmailCompleteSchema);
    const emailAuthRequest = await consumeCompletedRegistryEmailAuthRequestByToken(env, body.token);
    if (!emailAuthRequest) {
      const pendingRequest = await getRegistryEmailAuthRequestByToken(env, body.token);
      if (!pendingRequest) throw new HttpError("error.auth.registry_email.state.missing", 404);
      if (isExpired(pendingRequest.expires_at)) {
        await deleteRegistryEmailAuthRequest(env, pendingRequest.challenge_id);
        throw new HttpError("error.auth.registry_email.state.expired", 400);
      }
      if (!pendingRequest.session_token) throw new HttpError("error.auth.registry_email.state.pending", 409);
      throw new HttpError("error.auth.registry_email.state.missing", 404);
    }
    if (!emailAuthRequest.session_token) throw new HttpError("error.auth.registry_email.state.missing", 404);
    const session = await getAuthSession(env, emailAuthRequest.session_token);
    if (!session) throw new HttpError("error.auth.registry_email.session.missing", 404);
    if (isExpired(session.expires_at)) throw new HttpError("error.auth.registry_email.session.expired", 401);
    return jsonWithCors(request, env, authSessionResponseForEnv(env, session));
  }

  // GET /login/oidc/{provider} — server-side OIDC initiation with return_to
  if (method === "GET" && url.pathname.startsWith("/login/oidc/")) {
    const providerName = decodeURIComponent(url.pathname.slice("/login/oidc/".length));
    const challengeId = url.searchParams.get("challenge_id") ?? "";
    if (challengeId) {
      const challenge = await getChallenge(env, challengeId);
      if (!challenge) throw new HttpError("error.auth.challenge.unknown_id", 404);
      assertChallengeFresh(challenge);
    }
    const provider = oidcProviderByName(configuredOidcProviders(env), providerName);
    if (!provider) throw new HttpError(uiMessage("error.auth.oidc.provider.unknown", { provider: providerName }), 404);
    const discovery = await fetchOidcDiscovery(provider);
    const redirectUri = oidcCallbackUrl(env, providerName);
    const authorization = await createOidcAuthorizationRequest(provider, discovery, challengeId, redirectUri);
    const siteReturnUrl = returnToFromQuery(url, env);
    const lang = langFromQuery(url);
    const siteReturnUrlWithLang = lang ? `${siteReturnUrl}?lang=${encodeURIComponent(lang)}` : siteReturnUrl;
    await putOidcAuthRequest(env, { ...authorization.record, site_return_url: siteReturnUrlWithLang });
    return Response.redirect(authorization.authorizationUrl, 302);
  }

  // POST /v1/auth/oidc/{provider}/start (API variant, kept for autopeer compatibility)
  if (method === "POST" && url.pathname.startsWith("/v1/auth/oidc/") && url.pathname.endsWith("/start") && url.pathname.split("/").length === 6) {
    const providerName = decodeURIComponent(url.pathname.split("/")[4] ?? "");
    const body = await parseBody<OidcStartRequest>(request, OidcStartSchema);
    if (body.challenge_id) {
      const challenge = await getChallenge(env, body.challenge_id);
      if (!challenge) throw new HttpError("error.auth.challenge.unknown_id", 404);
      assertChallengeFresh(challenge);
    }
    const provider = oidcProviderByName(configuredOidcProviders(env), providerName);
    if (!provider) throw new HttpError(uiMessage("error.auth.oidc.provider.unknown", { provider: providerName }), 404);
    const discovery = await fetchOidcDiscovery(provider);
    const redirectUri = oidcCallbackUrl(env, providerName);
    const authorization = await createOidcAuthorizationRequest(provider, discovery, body.challenge_id ?? "", redirectUri);
    const siteReturnUrl = returnToFromRequestBody(body.return_to, env) ?? siteReturnUrlFromRequest(request, env);
    await putOidcAuthRequest(env, { ...authorization.record, site_return_url: siteReturnUrl });
    const response: OidcStartResponse = { authorization_url: authorization.authorizationUrl };
    return jsonWithCors(request, env, response);
  }

  // GET /oidc/callback/{provider}
  if (method === "GET" && url.pathname.startsWith(OIDC_CALLBACK_PREFIX)) {
    const providerName = decodeURIComponent(url.pathname.slice(OIDC_CALLBACK_PREFIX.length));
    if (!providerName) throw new HttpError("error.auth.oidc.callback.provider.missing", 400);

    const error = url.searchParams.get("error");
    if (error) {
      const description = url.searchParams.get("error_description");
      const message = uiMessage("error.auth.oidc.provider.rejected", { error, description: description ?? "" });
      // No auth request to look up yet — redirect to AUTH_SITE_URL as fallback
      return siteRedirectResponse(env.AUTH_SITE_URL, fragmentMessage("oidc_error", message));
    }

    const state = url.searchParams.get("state");
    const code = url.searchParams.get("code");
    if (!state || !code) {
      return siteRedirectResponse(env.AUTH_SITE_URL, fragmentMessage("oidc_error", uiMessage("error.auth.oidc.callback.params.missing")));
    }

    const provider = oidcProviderByName(configuredOidcProviders(env), providerName);
    if (!provider) {
      return siteRedirectResponse(env.AUTH_SITE_URL, fragmentMessage("oidc_error", uiMessage("error.auth.oidc.provider.unknown", { provider: providerName })));
    }

    const authRequest = await getOidcAuthRequest(env, state);
    if (!authRequest || authRequest.provider !== providerName) {
      return siteRedirectResponse(env.AUTH_SITE_URL, fragmentMessage("oidc_error", uiMessage("error.auth.oidc.state.missing")));
    }

    const returnUrl = authRequest.site_return_url ?? env.AUTH_SITE_URL;

    if (authRequest.session_token) {
      const existingSession = await getAuthSession(env, authRequest.session_token);
      if (existingSession && !isExpired(existingSession.expires_at)) {
        return siteRedirectResponse(returnUrl, sessionRedirectFragment(env, existingSession));
      }
      return siteRedirectResponse(returnUrl, `oidc_state=${encodeURIComponent(authRequest.state)}`);
    }

    if (isExpired(authRequest.expires_at)) {
      await deleteOidcAuthRequest(env, authRequest.state);
      return siteRedirectResponse(returnUrl, fragmentMessage("oidc_error", uiMessage("error.auth.oidc.state.expired")));
    }

    let challenge = null;
    if (authRequest.challenge_id) {
      challenge = await getChallenge(env, authRequest.challenge_id);
      if (!challenge) {
        await deleteOidcAuthRequest(env, authRequest.state);
        return siteRedirectResponse(returnUrl, fragmentMessage("oidc_error", uiMessage("error.auth.challenge.expired")));
      }
    }

    try {
      if (challenge) assertChallengeFresh(challenge);
      const discovery = await fetchOidcDiscovery(provider);
      const tokenResponse = await exchangeAuthorizationCode(env, provider, discovery, code, authRequest.redirect_uri, authRequest.code_verifier);
      const claimSources = await verifiedOidcClaimSources(tokenResponse, provider, discovery, authRequest.nonce);
      const tokenAsn = normalizeSupportedAutopeerAsn(oidcAsnFromClaimSources(claimSources, provider));
      let session: SessionRecord;

      if (challenge) {
        if (tokenAsn !== challenge.asn) {
          throw new HttpError(uiMessage("error.auth.oidc.identity.asn_mismatch", { token_asn: tokenAsn, requested_asn: challenge.asn }), 400);
        }
        const effectiveMnt = oidcMaintainerFromClaimSources(claimSources, provider, challenge.maintainers);
        session = sessionFromOidcIdentity(provider, challenge.asn, effectiveMnt);
      } else {
        const maintainers = await loadMaintainersForRequestAsn(env, tokenAsn);
        const effectiveMnt = oidcMaintainerFromClaimSources(claimSources, provider, maintainers);
        session = sessionFromOidcIdentity(provider, tokenAsn, effectiveMnt);
      }

      await putAuthSession(env, session);
      await putOidcAuthRequest(env, { ...authRequest, session_token: session.token });
      if (challenge) await deleteChallenge(env, challenge.id);
      return siteRedirectResponse(returnUrl, sessionRedirectFragment(env, session));
    } catch (callbackError) {
      await deleteOidcAuthRequest(env, authRequest.state);
      const message = errorMessage(callbackError, "error.auth.oidc.callback.failed");
      return siteRedirectResponse(returnUrl, fragmentMessage("oidc_error", message));
    }
  }

  // GET /auth/email/callback
  if (method === "GET" && url.pathname === REGISTRY_EMAIL_CALLBACK_PATH) {
    const challengeId = url.searchParams.get("challenge_id");
    const token = url.searchParams.get("token");
    if (!challengeId || !token) {
      return siteRedirectResponse(env.AUTH_SITE_URL, fragmentMessage("email_error", uiMessage("error.auth.registry_email.callback.params.missing")));
    }

    const emailAuthRequest = await getRegistryEmailAuthRequestByToken(env, token);
    if (!emailAuthRequest || emailAuthRequest.challenge_id !== challengeId) {
      return siteRedirectResponse(env.AUTH_SITE_URL, fragmentMessage("email_error", uiMessage("error.auth.registry_email.state.missing")));
    }

    const returnUrl = emailAuthRequest.site_return_url ?? env.AUTH_SITE_URL;

    if (emailAuthRequest.session_token) {
      const existingSession = await getAuthSession(env, emailAuthRequest.session_token);
      if (existingSession && !isExpired(existingSession.expires_at)) {
        return siteRedirectResponse(returnUrl, sessionRedirectFragment(env, existingSession));
      }
    }

    if (isExpired(emailAuthRequest.expires_at)) {
      await deleteRegistryEmailAuthRequest(env, emailAuthRequest.challenge_id);
      return siteRedirectResponse(returnUrl, fragmentMessage("email_error", uiMessage("error.auth.registry_email.state.expired")));
    }

    try {
      const session = await createCompletedRegistryEmailSession(env, challengeId, emailAuthRequest.effective_mnt);
      await putRegistryEmailAuthRequest(env, { ...emailAuthRequest, session_token: session.token });
      return siteRedirectResponse(returnUrl, sessionRedirectFragment(env, session));
    } catch (callbackError) {
      await deleteRegistryEmailAuthRequest(env, emailAuthRequest.challenge_id);
      const message = errorMessage(callbackError, "error.auth.registry_email.callback.failed");
      return siteRedirectResponse(returnUrl, fragmentMessage("email_error", message));
    }
  }

  // POST /v1/auth/oidc/complete
  if (method === "POST" && url.pathname === "/v1/auth/oidc/complete") {
    const body = await parseBody<OidcCompleteRequest>(request, OidcCompleteSchema);
    const authRequest = await getOidcAuthRequest(env, body.state);
    if (!authRequest) throw new HttpError("error.auth.oidc.state.missing", 404);
    if (!authRequest.session_token && isExpired(authRequest.expires_at)) {
      await deleteOidcAuthRequest(env, authRequest.state);
      throw new HttpError("error.auth.oidc.state.expired", 400);
    }
    if (!authRequest.session_token) throw new HttpError("error.auth.oidc.state.pending", 409);
    const session = await getAuthSession(env, authRequest.session_token);
    await deleteOidcAuthRequest(env, authRequest.state);
    if (!session) throw new HttpError("error.auth.oidc.session.missing", 404);
    if (isExpired(session.expires_at)) throw new HttpError("error.auth.oidc.session.expired", 401);
    return jsonWithCors(request, env, authSessionResponseForEnv(env, session));
  }

  throw new HttpError("error.request.route.not_found", 404);
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      return await router(request, env);
    } catch (error) {
      if (error instanceof HttpError) {
        const publicMessage = isUiMessageKey(error.uiMessage.key) ? error.uiMessage : uiMessage(error.uiMessage.key);
        return errorWithCors(request, env, publicMessage, error.status);
      }
      if (error instanceof I18nError) {
        return errorWithCors(request, env, error.uiMessage, 500);
      }
      console.error("auth-worker request failed", error);
      return errorWithCors(request, env, uiMessage("error.internal"), 500);
    }
  },
} satisfies ExportedHandler<Env>;
