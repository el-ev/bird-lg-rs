import type { ApiError, UiMessage } from "./types";

const ASN_PATTERN = /^424242\d+$/;
export const UNSUPPORTED_ASN_RANGE_MESSAGE = "error.auth.asn.unsupported";

export function isUiMessageKey(value: string): boolean {
  return /^[a-z0-9_.-]+$/u.test(value) && value.includes(".");
}

export function uiMessage(key: string, params?: Record<string, string>): UiMessage {
  const normalizedParams = params && Object.keys(params).length > 0 ? params : undefined;
  return {
    key,
    ...(normalizedParams ? { params: normalizedParams } : {}),
  };
}

export function toUiMessage(message: string | UiMessage): UiMessage {
  if (typeof message !== "string") {
    return message;
  }
  return uiMessage(message);
}

export class HttpError extends Error {
  readonly uiMessage: UiMessage;

  constructor(
    message: string | UiMessage,
    readonly status: number,
  ) {
    const ui = toUiMessage(message);
    super(ui.key);
    this.uiMessage = ui;
  }
}

export class I18nError extends Error {
  constructor(public readonly uiMessage: UiMessage) {
    super(uiMessage.key);
  }
}

export function nowIso(): string {
  return new Date().toISOString();
}

export function isExpired(iso: string): boolean {
  return Date.parse(iso) <= Date.now();
}

export function addSeconds(iso: string, seconds: number): string {
  return new Date(Date.parse(iso) + seconds * 1000).toISOString();
}

export function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  return new Response(JSON.stringify(body, null, 2), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      ...headers,
    },
  });
}

export function errorResponse(
  message: string | UiMessage,
  status = 400,
  headers?: HeadersInit,
): Response {
  const body: ApiError = { error: toUiMessage(message) };
  return jsonResponse(body, status, headers);
}

export async function readJson<T>(request: Request): Promise<T> {
  try {
    return (await request.json()) as T;
  } catch {
    throw new HttpError("error.request.body.invalid_json", 400);
  }
}

export function bearerToken(request: Request): string | null {
  const header = request.headers.get("authorization");
  if (!header) {
    return null;
  }

  const [scheme, token] = header.split(/\s+/, 2);
  if (!scheme || !token || scheme.toLowerCase() !== "bearer") {
    return null;
  }

  return token.trim();
}

export function requireNonEmptyString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new HttpError(uiMessage("error.field.required", { field }), 400);
  }
  return value.trim();
}

export function requireOptionalString(value: unknown, field: string): string | null {
  if (value === undefined || value === null) {
    return null;
  }
  if (typeof value !== "string") {
    throw new HttpError(uiMessage("error.field.must_be_string", { field }), 400);
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function requireBoolean(value: unknown, field: string): boolean {
  if (typeof value !== "boolean") {
    throw new HttpError(uiMessage("error.field.must_be_boolean", { field }), 400);
  }
  return value;
}

export function requireOptionalInteger(value: unknown, field: string): number | null {
  if (value === undefined || value === null) {
    return null;
  }
  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw new HttpError(uiMessage("error.field.must_be_integer", { field }), 400);
  }
  return value;
}

export function requireRecord(value: unknown, field: string): Record<string, unknown> {
  if (!isTruthyRecord(value)) {
    throw new HttpError(uiMessage("error.field.must_be_object", { field }), 400);
  }
  return value;
}

export function normalizeAsn(
  raw: string,
  errorKey = "error.asn.format",
): string {
  const asn = raw.trim().toUpperCase().replace(/^AS/, "");
  if (!ASN_PATTERN.test(asn)) {
    throw new HttpError(uiMessage(errorKey), 400);
  }
  return asn;
}

export function normalizeSupportedAutopeerAsn(raw: string): string {
  return normalizeAsn(raw, UNSUPPORTED_ASN_RANGE_MESSAGE);
}

export function defaultPeerPort(asn: string): number {
  return Number(asn.slice(-5));
}

export function isTruthyRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function toBase64(input: string): string {
  const bytes = new TextEncoder().encode(input);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

export function fromBase64(input: string): string {
  const binary = atob(input.replace(/\s+/g, ""));
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

export function joinPath(...parts: string[]): string {
  return parts
    .flatMap((part) => part.split("/"))
    .filter(Boolean)
    .map((part) => encodeURIComponent(part))
    .join("/");
}

export function buildCorsHeaders(request: Request): Headers {
  const headers = new Headers();
  headers.set("access-control-allow-methods", "GET,POST,PATCH,DELETE,OPTIONS");
  headers.set("access-control-allow-headers", "authorization,content-type");
  headers.set("access-control-max-age", "86400");

  const origin = request.headers.get("origin");
  if (origin) {
    headers.set("access-control-allow-origin", origin);
    headers.set("vary", "origin");
  } else {
    headers.set("access-control-allow-origin", "*");
  }

  return headers;
}

export function jsonWithCors(request: Request, body: unknown, status = 200): Response {
  return jsonResponse(body, status, buildCorsHeaders(request));
}

export function errorWithCors(
  request: Request,
  message: string | UiMessage,
  status = 400,
): Response {
  return errorResponse(message, status, buildCorsHeaders(request));
}

const OPERATOR_HINT_MARKERS = [
  " Hint: GitHub requires a valid User-Agent header on all API requests.",
];

export function stripOperatorHints(message: string): string {
  let stripped = message;

  for (const marker of OPERATOR_HINT_MARKERS) {
    const index = stripped.indexOf(marker);
    if (index >= 0) {
      stripped = stripped.slice(0, index);
    }
  }

  stripped = stripped.trim();
  return stripped.length > 0 ? stripped : "internal error";
}

export function parseJsonEnv<T>(raw: string, field: string): T {
  try {
    return JSON.parse(raw) as T;
  } catch (error) {
    throw new Error(`${field} must be valid JSON: ${String(error)}`);
  }
}

export function isTerminalOperationState(state: string): boolean {
  return state === "completed" || state === "failed" || state === "conflict";
}

export function readNamedSecret(env: Env, name: string): string {
  const value = (env as unknown as Record<string, unknown>)[name];
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${name} is not configured`);
  }
  return value;
}

export function readOptionalEnvString(env: Env, name: string): string | null {
  const value = (env as unknown as Record<string, unknown>)[name];
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function readSecret(
  env: Env,
  name: "GITHUB_TOKEN" | "DN42_GIT_TOKEN" | "RESEND_API_KEY",
): string {
  return readNamedSecret(env, name);
}

export function readOptionalSecret(
  env: Env,
  name: "ANSIBLE_VAULT_PASSWORD",
): string | null {
  const value = (env as unknown as Record<string, unknown>)[name];
  return typeof value === "string" && value.length > 0 ? value : null;
}

export function timingSafeEqual(left: string, right: string): boolean {
  const leftBytes = new TextEncoder().encode(left);
  const rightBytes = new TextEncoder().encode(right);
  const maxLength = Math.max(leftBytes.length, rightBytes.length);
  let diff = leftBytes.length ^ rightBytes.length;

  for (let index = 0; index < maxLength; index += 1) {
    diff |= (leftBytes[index] ?? 0) ^ (rightBytes[index] ?? 0);
  }

  return diff === 0;
}

export function randomBase64Url(byteLength = 32): string {
  const bytes = crypto.getRandomValues(new Uint8Array(byteLength));
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/u, "");
}

export function parseConfiguredAsns(raw: string): Set<string> {
  return new Set(
    raw
      .split(/[,\s]+/)
      .map((value) => value.trim())
      .filter(Boolean)
      .map((value) => normalizeAsn(value)),
  );
}
