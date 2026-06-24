import type { UiMessage } from "./types";

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

export function readNamedSecret(env: object, name: string): string {
  const value = (env as Record<string, unknown>)[name];
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${name} is not configured`);
  }
  return value;
}

export function readOptionalEnvString(env: object, name: string): string | null {
  const value = (env as Record<string, unknown>)[name];
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function readSecret(
  env: object,
  name: string,
): string {
  return readNamedSecret(env, name);
}

export function readOptionalSecret(
  env: object,
  name: string,
): string | null {
  const value = (env as Record<string, unknown>)[name];
  return typeof value === "string" && value.length > 0 ? value : null;
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
