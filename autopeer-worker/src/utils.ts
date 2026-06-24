import type { ZodType, core } from "zod";
import {
  HttpError,
  uiMessage,
  toUiMessage,
} from "dn42-auth-worker/utils";
import type { ApiError, UiMessage } from "./types";

export {
  HttpError,
  I18nError,
  UNSUPPORTED_ASN_RANGE_MESSAGE,
  uiMessage,
  toUiMessage,
  isUiMessageKey,
  nowIso,
  isExpired,
  bearerToken,
  normalizeAsn,
  normalizeSupportedAutopeerAsn,
  readOptionalEnvString,
  readSecret,
  readOptionalSecret,
  parseConfiguredAsns,
  toBase64,
  joinPath,
} from "dn42-auth-worker/utils";

const SECURITY_HEADERS = {
  "x-content-type-options": "nosniff",
  "x-frame-options": "DENY",
  "referrer-policy": "strict-origin-when-cross-origin",
};

export function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  return new Response(JSON.stringify(body, null, 2), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      ...SECURITY_HEADERS,
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

export async function parseBody<T>(
  request: Request,
  schema: ZodType<T, unknown>,
  rootLabel = "request body",
): Promise<T> {
  const raw = await readJson<unknown>(request);
  const result = schema.safeParse(raw);
  if (result.success) {
    return result.data;
  }
  throw issueToHttpError(result.error.issues[0]!, rootLabel);
}

function issueToHttpError(issue: core.$ZodIssue, rootLabel: string): HttpError {
  const field = issue.path.length === 0
    ? rootLabel
    : issue.path.map((segment) => String(segment)).join(".");

  if (issue.code === "custom") {
    const params = (issue.params ?? {}) as { uiKey?: unknown; literal?: unknown };
    if (typeof params.uiKey === "string") {
      if (params.literal === true) {
        return new HttpError(params.uiKey, 400);
      }
      return new HttpError(uiMessage(params.uiKey, { field }), 400);
    }
  }

  if (issue.code === "invalid_type") {
    switch (issue.expected) {
      case "string":
        return new HttpError(uiMessage("error.field.must_be_string", { field }), 400);
      case "boolean":
        return new HttpError(uiMessage("error.field.must_be_boolean", { field }), 400);
      case "int":
      case "number":
        return new HttpError(uiMessage("error.field.must_be_integer", { field }), 400);
      case "object":
        return new HttpError(uiMessage("error.field.must_be_object", { field }), 400);
      case "array":
        return new HttpError(uiMessage("error.field.must_be_array", { field }), 400);
    }
  }

  return new HttpError(uiMessage("error.field.required", { field }), 400);
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

export function isTerminalOperationState(state: string): boolean {
  return state === "completed" || state === "failed" || state === "conflict";
}

export function defaultPeerPort(asn: string): number {
  return Number(asn.slice(-5));
}

export function isTruthyRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
