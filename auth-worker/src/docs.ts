export const OPENAPI_PATH = "/openapi.json";
export const SWAGGER_PATH = "/swagger";

type JsonObject = Record<string, unknown>;

function ref(name: string): JsonObject {
  return { $ref: `#/components/schemas/${name}` };
}

function jsonContent(schema: JsonObject): JsonObject {
  return {
    "application/json": {
      schema,
    },
  };
}

function jsonRequestBody(schema: JsonObject, description?: string): JsonObject {
  return {
    required: true,
    ...(description ? { description } : {}),
    content: jsonContent(schema),
  };
}

function jsonResponse(description: string, schema: JsonObject): JsonObject {
  return {
    description,
    content: jsonContent(schema),
  };
}

function errorResponse(description: string): JsonObject {
  return jsonResponse(description, ref("ApiError"));
}

function redirectResponse(description: string): JsonObject {
  return {
    description,
    headers: {
      Location: {
        description: "Redirect target URL",
        schema: { type: "string", format: "uri" },
      },
    },
  };
}

function pathParam(name: string, description: string): JsonObject {
  return {
    name,
    in: "path",
    required: true,
    description,
    schema: { type: "string" },
  };
}

function queryParam(name: string, description: string, required = false): JsonObject {
  return {
    name,
    in: "query",
    required,
    description,
    schema: { type: "string" },
  };
}

function bearerSecurity(): JsonObject[] {
  return [{ bearerAuth: [] }];
}

const components = {
  securitySchemes: {
    bearerAuth: {
      type: "http",
      scheme: "bearer",
      bearerFormat: "session_token",
      description: "Use the session_token returned by a completed auth flow.",
    },
  },
  schemas: {
    UiMessage: {
      type: "object",
      required: ["key"],
      properties: {
        key: { type: "string" },
        params: {
          type: "object",
          additionalProperties: { type: "string" },
        },
      },
    },
    ApiError: {
      type: "object",
      required: ["error"],
      properties: {
        error: ref("UiMessage"),
      },
    },
    RegistryEmailTarget: {
      type: "object",
      required: ["maintainer", "emails"],
      properties: {
        maintainer: { type: "string" },
        emails: {
          type: "array",
          items: { type: "string", format: "email" },
        },
      },
    },
    AuthMethod: {
      type: "object",
      required: ["kind", "label", "description"],
      properties: {
        kind: {
          type: "string",
          enum: [
            "registry_ssh",
            "registry_pgp",
            "registry_email",
            "oidc",
            "host_impersonation",
          ],
        },
        label: ref("UiMessage"),
        description: ref("UiMessage"),
        provider: { type: "string" },
        ssh_fingerprints: {
          type: "array",
          items: { type: "string" },
        },
        pgp_fingerprints: {
          type: "array",
          items: { type: "string" },
        },
        email_targets: {
          type: "array",
          items: ref("RegistryEmailTarget"),
        },
      },
    },
    RuntimeConfigResponse: {
      type: "object",
      required: ["autopeer_api_url", "oidc_methods", "allowed_return_urls"],
      properties: {
        autopeer_api_url: { type: "string", format: "uri" },
        oidc_methods: {
          type: "array",
          items: ref("AuthMethod"),
        },
        allowed_return_urls: {
          type: "array",
          items: { type: "string", format: "uri" },
        },
      },
    },
    HealthResponse: {
      type: "object",
      required: ["ok", "now"],
      properties: {
        ok: { type: "boolean" },
        now: { type: "string", format: "date-time" },
        registry: ref("RegistryHealth"),
      },
    },
    RegistryHealth: {
      type: "object",
      required: ["ok"],
      properties: {
        ok: { type: "boolean" },
        reason: {
          type: "string",
          enum: [
            "token_rejected",
            "access_forbidden",
            "repo_not_visible",
            "branch_missing",
            "request_failed",
          ],
        },
        status: { type: "integer", description: "HTTP status the registry API returned (0 = network failure)" },
      },
    },
    AuthStartRequest: {
      type: "object",
      required: ["asn"],
      properties: {
        asn: { type: "string", example: "AS4242420000" },
      },
    },
    AuthStartResponse: {
      type: "object",
      required: ["asn", "challenge_id", "challenge_text", "challenge_ttl_seconds", "methods"],
      properties: {
        asn: { type: "string" },
        challenge_id: { type: "string" },
        challenge_text: { type: "string" },
        challenge_ttl_seconds: { type: "integer" },
        methods: {
          type: "array",
          items: ref("AuthMethod"),
        },
      },
    },
    HostImpersonationRequest: {
      type: "object",
      required: ["asn"],
      properties: {
        asn: { type: "string", example: "AS4242420000" },
        effective_mnt: { type: "string" },
      },
    },
    RegistrySshVerifyRequest: {
      type: "object",
      required: ["challenge_id", "signature"],
      properties: {
        challenge_id: { type: "string" },
        signature: { type: "string" },
      },
    },
    RegistryPgpVerifyRequest: {
      type: "object",
      required: ["challenge_id", "public_key", "signed_message"],
      properties: {
        challenge_id: { type: "string" },
        public_key: { type: "string" },
        signed_message: { type: "string" },
      },
    },
    PgpKeyLookupResponse: {
      type: "object",
      required: ["fingerprint", "found"],
      properties: {
        fingerprint: { type: "string" },
        found: { type: "boolean" },
        public_key: { type: "string" },
        source: { type: "string" },
      },
    },
    RegistryEmailSendRequest: {
      type: "object",
      required: ["challenge_id"],
      properties: {
        challenge_id: { type: "string" },
        effective_mnt: { type: "string" },
        locale: {
          type: "string",
          description: "Preferred locale for the verification email.",
        },
      },
    },
    RegistryEmailSendResponse: {
      type: "object",
      required: ["effective_mnt", "emails", "expires_at"],
      properties: {
        effective_mnt: { type: "string" },
        emails: {
          type: "array",
          items: { type: "string", format: "email" },
        },
        expires_at: { type: "string", format: "date-time" },
      },
    },
    RegistryEmailVerifyRequest: {
      type: "object",
      required: ["challenge_id", "code"],
      properties: {
        challenge_id: { type: "string" },
        code: { type: "string" },
      },
    },
    RegistryEmailCompleteRequest: {
      type: "object",
      required: ["token"],
      properties: {
        token: { type: "string" },
      },
    },
    AuthSessionResponse: {
      type: "object",
      required: [
        "session_token",
        "asn",
        "effective_mnt",
        "auth_method",
        "can_impersonate",
        "expires_at",
      ],
      properties: {
        session_token: { type: "string" },
        asn: { type: "string" },
        effective_mnt: { type: "string" },
        auth_method: ref("AuthMethod"),
        can_impersonate: { type: "boolean" },
        expires_at: { type: "string", format: "date-time" },
      },
    },
  },
} satisfies JsonObject;

const paths = {
  "/config.json": {
    get: {
      tags: ["meta"],
      summary: "Load auth frontend runtime configuration",
      responses: {
        "200": jsonResponse("Current auth runtime configuration", ref("RuntimeConfigResponse")),
      },
    },
  },
  "/health": {
    get: {
      tags: ["meta"],
      summary: "Worker health check",
      parameters: [
        queryParam("deep", "Set to 1 to also verify the DN42 registry is readable with the configured token"),
      ],
      responses: {
        "200": jsonResponse("Health status", ref("HealthResponse")),
        "503": jsonResponse("Registry is not readable (deep check only)", ref("HealthResponse")),
      },
    },
  },
  "/v1/auth/start": {
    post: {
      tags: ["auth"],
      summary: "Start a registry-based auth challenge",
      requestBody: jsonRequestBody(ref("AuthStartRequest")),
      responses: {
        "200": jsonResponse("Challenge created", ref("AuthStartResponse")),
        "400": errorResponse("Invalid ASN or no supported auth methods"),
        "502": errorResponse("Registry lookup failed or registry unavailable to the worker"),
      },
    },
  },
  "/v1/auth/impersonate": {
    post: {
      tags: ["auth"],
      summary: "Open a host-ASN impersonation session",
      security: bearerSecurity(),
      requestBody: jsonRequestBody(ref("HostImpersonationRequest")),
      responses: {
        "200": jsonResponse("Impersonation session created", ref("AuthSessionResponse")),
        "400": errorResponse("Invalid target ASN or maintainer"),
        "403": errorResponse("Caller cannot impersonate other ASNs"),
        "502": errorResponse("Registry lookup failed"),
      },
    },
  },
  "/v1/auth/verify/registry-ssh": {
    post: {
      tags: ["auth"],
      summary: "Redeem a challenge with a registry SSH signature",
      requestBody: jsonRequestBody(ref("RegistrySshVerifyRequest")),
      responses: {
        "200": jsonResponse("Session created", ref("AuthSessionResponse")),
        "400": errorResponse("Challenge expired or signature is invalid"),
        "404": errorResponse("Challenge was not found"),
      },
    },
  },
  "/v1/auth/verify/registry-pgp": {
    post: {
      tags: ["auth"],
      summary: "Redeem a challenge with a registry OpenPGP signature",
      requestBody: jsonRequestBody(ref("RegistryPgpVerifyRequest")),
      responses: {
        "200": jsonResponse("Session created", ref("AuthSessionResponse")),
        "400": errorResponse("Challenge expired or signed message is invalid"),
        "404": errorResponse("Challenge was not found"),
      },
    },
  },
  "/v1/auth/lookup/pgp-key": {
    get: {
      tags: ["auth"],
      summary: "Look up an OpenPGP key by fingerprint",
      parameters: [
        queryParam("fingerprint", "OpenPGP fingerprint to normalize and look up", true),
      ],
      responses: {
        "200": jsonResponse("PGP key lookup result", ref("PgpKeyLookupResponse")),
        "400": errorResponse("Fingerprint is invalid"),
      },
    },
  },
  "/v1/auth/verify/registry-email/send": {
    post: {
      tags: ["auth"],
      summary: "Send a registry email auth link and code",
      requestBody: jsonRequestBody(ref("RegistryEmailSendRequest")),
      responses: {
        "200": jsonResponse("Email challenge sent", ref("RegistryEmailSendResponse")),
        "400": errorResponse("Request is invalid or challenge has expired"),
        "404": errorResponse("Challenge was not found"),
        "503": errorResponse("Registry email auth is not configured"),
      },
    },
  },
  "/v1/auth/verify/registry-email": {
    post: {
      tags: ["auth"],
      summary: "Redeem a registry email code",
      requestBody: jsonRequestBody(ref("RegistryEmailVerifyRequest")),
      responses: {
        "200": jsonResponse("Session created", ref("AuthSessionResponse")),
        "400": errorResponse("Code is invalid or the challenge has expired"),
        "404": errorResponse("Email auth state was not found"),
        "409": errorResponse("Email login already completed or still pending"),
      },
    },
  },
  "/v1/auth/verify/registry-email/complete": {
    post: {
      tags: ["auth"],
      summary: "Redeem a completed registry email login token",
      requestBody: jsonRequestBody(ref("RegistryEmailCompleteRequest")),
      responses: {
        "200": jsonResponse("Session created", ref("AuthSessionResponse")),
        "400": errorResponse("Email auth request expired"),
        "401": errorResponse("Completed session has expired"),
        "404": errorResponse("Token or session was not found"),
        "409": errorResponse("Email auth is still pending"),
      },
    },
  },
  "/login/oidc/{provider}": {
    get: {
      tags: ["browser auth"],
      summary: "Start an OIDC browser redirect flow",
      parameters: [
        pathParam("provider", "Configured OIDC provider name"),
        queryParam("return_to", "Allowed consumer site origin to receive the completed session"),
        queryParam("challenge_id", "Optional registry challenge to bind the OIDC login to"),
        queryParam("lang", "Optional frontend language code"),
      ],
      responses: {
        "302": redirectResponse("Redirects to the configured OIDC provider"),
        "400": errorResponse("Challenge is invalid or expired"),
        "404": errorResponse("Challenge or provider was not found"),
      },
    },
  },
  "/oidc/callback/{provider}": {
    get: {
      tags: ["callbacks"],
      summary: "Handle the OIDC provider callback",
      parameters: [
        pathParam("provider", "Configured OIDC provider name"),
      ],
      responses: {
        "302": redirectResponse(
          "Redirects to an allowed consumer site with auth_session or oidc_error in the fragment",
        ),
      },
    },
  },
  "/auth/email/callback": {
    get: {
      tags: ["callbacks"],
      summary: "Handle the emailed registry auth callback",
      parameters: [
        queryParam("challenge_id", "Registry challenge identifier", true),
        queryParam("token", "One-time email callback token", true),
      ],
      responses: {
        "302": redirectResponse(
          "Redirects to an allowed consumer site with auth_session or email_error in the fragment",
        ),
      },
    },
  },
} satisfies JsonObject;

export function openApiSpec(request: Request, authSiteUrl?: string): JsonObject {
  const origin = new URL(request.url).origin;
  return {
    openapi: "3.1.0",
    info: {
      title: "dn42 auth worker API",
      version: "0.1.0",
      description:
        "Central authentication API for dn42 consumer sites. Browser flows redirect back only to configured allowed return URLs; protected endpoints use bearer session_token.",
    },
    servers: [
      {
        url: authSiteUrl || origin,
        description: "Configured auth API base URL",
      },
    ],
    tags: [
      { name: "meta", description: "Runtime config and health endpoints" },
      { name: "auth", description: "Challenge creation, verification, and session endpoints" },
      { name: "browser auth", description: "Browser redirect initiation routes" },
      { name: "callbacks", description: "Provider callback routes that redirect back to allowed consumer sites" },
    ],
    paths,
    components,
  };
}

export function swaggerUiHtml(specPath = OPENAPI_PATH): string {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>dn42 Auth API Docs</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
    <style>
      html, body {
        margin: 0;
        padding: 0;
        background: #f5f7fb;
      }
      #swagger-ui {
        max-width: 1200px;
        margin: 0 auto;
      }
    </style>
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.ui = SwaggerUIBundle({
        url: ${JSON.stringify(specPath)},
        dom_id: "#swagger-ui",
        deepLinking: true,
        docExpansion: "list",
        displayRequestDuration: true,
        persistAuthorization: true
      });
    </script>
  </body>
</html>`;
}
