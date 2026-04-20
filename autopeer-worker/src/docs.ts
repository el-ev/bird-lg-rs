import { readOptionalEnvString } from "./utils";

export const OPENAPI_PATH = "/openapi.json";
export const SWAGGER_PATH = "/swagger";

type JsonObject = Record<string, unknown>;

function ref(name: string): JsonObject {
  return { $ref: `#/components/schemas/${name}` };
}

function nullable(schema: JsonObject): JsonObject {
  return { anyOf: [schema, { type: "null" }] };
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

function bearerSecurity(): JsonObject[] {
  return [{ bearerAuth: [] }];
}

function apiServerUrl(request: Request, env: Env): string {
  return readOptionalEnvString(env, "AUTOPEER_URL") ?? new URL(request.url).origin;
}

const components = {
  securitySchemes: {
    bearerAuth: {
      type: "http",
      scheme: "bearer",
      bearerFormat: "session_token",
      description: "Use the session_token returned by an auth endpoint.",
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
        fallback: nullable({ type: "string" }),
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
      required: ["autopeer_url", "autopeer_site_url", "oidc_methods"],
      properties: {
        autopeer_url: { type: "string", format: "uri" },
        autopeer_site_url: { type: "string", format: "uri" },
        looking_glass_url: { type: "string", format: "uri" },
        oidc_methods: {
          type: "array",
          items: ref("AuthMethod"),
        },
      },
    },
    HealthResponse: {
      type: "object",
      required: ["ok", "now"],
      properties: {
        ok: { type: "boolean" },
        now: { type: "string", format: "date-time" },
      },
    },
    AuthStartRequest: {
      type: "object",
      required: ["asn"],
      properties: {
        asn: { type: "string" },
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
    RegistryEmailSendRequest: {
      type: "object",
      required: ["challenge_id"],
      properties: {
        challenge_id: { type: "string" },
        effective_mnt: { type: "string" },
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
    OidcStartRequest: {
      type: "object",
      properties: {
        challenge_id: { type: "string" },
      },
    },
    OidcStartResponse: {
      type: "object",
      required: ["authorization_url"],
      properties: {
        authorization_url: { type: "string", format: "uri" },
      },
    },
    OidcCompleteRequest: {
      type: "object",
      required: ["state"],
      properties: {
        state: { type: "string" },
      },
    },
    HostImpersonationRequest: {
      type: "object",
      required: ["asn"],
      properties: {
        asn: { type: "string" },
        effective_mnt: { type: "string" },
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
    PeeringInfo: {
      type: "object",
      properties: {
        ipv4: { type: "string" },
        ipv6: { type: "string" },
        link_local_ipv6: { type: "string" },
        wg_pubkey: { type: "string" },
        endpoint: { type: "string" },
        comment: { type: "string" },
      },
    },
    NodeView: {
      type: "object",
      required: ["name", "ip_support"],
      properties: {
        name: { type: "string" },
        endpoint_host: { type: "string" },
        region: { type: "string" },
        country: { type: "string" },
        ip_support: { type: "string" },
        comment: { type: "string" },
        peering: ref("PeeringInfo"),
        autopeer: { type: "boolean" },
      },
    },
    SessionMetadata: {
      type: "object",
      required: ["managed"],
      properties: {
        managed: { type: "boolean" },
        effective_mnt: { type: "string" },
        auth_provider: { type: "string" },
      },
    },
    PeerSessionSpec: {
      type: "object",
      required: [
        "endpoint",
        "wg_public_key",
        "ipv4",
        "ipv6",
        "extended_next_hop",
        "mp_bgp",
        "peering_strategy",
      ],
      properties: {
        comment: { type: "string" },
        endpoint: { type: "string" },
        wg_public_key: { type: "string" },
        port: { type: "integer" },
        peer4: { type: "string" },
        peer6: { type: "string" },
        own6: { type: "string" },
        keepalive: { type: "integer" },
        mtu: { type: "integer" },
        ipv4: { type: "boolean" },
        ipv6: { type: "boolean" },
        extended_next_hop: { type: "boolean" },
        mp_bgp: { type: "boolean" },
        peering_strategy: {
          type: "string",
          enum: ["full_table", "transit", "peer", "downstream"],
        },
      },
    },
    SessionView: {
      type: "object",
      required: ["node", "asn", "state"],
      properties: {
        node: { type: "string" },
        asn: { type: "string" },
        state: {
          type: "string",
          enum: ["managed", "manual", "pending_pr", "conflict"],
        },
        spec: ref("PeerSessionSpec"),
        metadata: ref("SessionMetadata"),
        pending_operation_id: { type: "string" },
        pull_request_url: { type: "string", format: "uri" },
        message: ref("UiMessage"),
      },
    },
    SessionListResponse: {
      type: "object",
      required: ["asn", "effective_mnt", "auth_method", "nodes", "sessions"],
      properties: {
        asn: { type: "string" },
        effective_mnt: { type: "string" },
        auth_method: ref("AuthMethod"),
        nodes: {
          type: "array",
          items: ref("NodeView"),
        },
        sessions: {
          type: "array",
          items: ref("SessionView"),
        },
      },
    },
    CreateSessionRequest: {
      type: "object",
      required: ["node", "session"],
      properties: {
        node: { type: "string" },
        session: ref("PeerSessionSpec"),
      },
    },
    UpdateSessionRequest: {
      type: "object",
      required: ["session"],
      properties: {
        session: ref("PeerSessionSpec"),
      },
    },
    OperationFailureDetails: {
      type: "object",
      required: ["stage"],
      properties: {
        stage: {
          type: "string",
          enum: ["checks", "preflight", "apply", "merge"],
        },
        step: nullable({ type: "string" }),
        conclusion: nullable({ type: "string" }),
        run_url: nullable({ type: "string", format: "uri" }),
        annotation: nullable({ type: "string" }),
      },
    },
    OperationStatus: {
      type: "object",
      required: [
        "id",
        "asn",
        "node",
        "kind",
        "state",
        "branch",
        "created_at",
        "updated_at",
      ],
      properties: {
        id: { type: "string" },
        asn: { type: "string" },
        node: { type: "string" },
        kind: {
          type: "string",
          enum: ["create", "update", "delete", "migrate"],
        },
        state: {
          type: "string",
          enum: [
            "pending_pull_request",
            "pending_checks",
            "applying",
            "pending_merge",
            "completed",
            "failed",
            "conflict",
          ],
        },
        branch: { type: "string" },
        pr_number: nullable({ type: "integer" }),
        pull_request_url: nullable({ type: "string", format: "uri" }),
        workflow_run_url: nullable({ type: "string", format: "uri" }),
        message: nullable(ref("UiMessage")),
        failure_details: nullable(ref("OperationFailureDetails")),
        created_at: { type: "string", format: "date-time" },
        updated_at: { type: "string", format: "date-time" },
      },
    },
  },
} satisfies JsonObject;

const paths = {
  "/config.json": {
    get: {
      tags: ["meta"],
      summary: "Load runtime frontend configuration",
      responses: {
        "200": jsonResponse(
          "Current frontend runtime configuration",
          ref("RuntimeConfigResponse"),
        ),
      },
    },
  },
  "/health": {
    get: {
      tags: ["meta"],
      summary: "Worker health check",
      responses: {
        "200": jsonResponse("Health status", ref("HealthResponse")),
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
        "502": errorResponse("Registry lookup failed"),
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
  "/v1/auth/oidc/{provider}/start": {
    post: {
      tags: ["auth"],
      summary: "Start an OIDC login flow",
      parameters: [
        pathParam("provider", "Configured OIDC provider name"),
      ],
      requestBody: jsonRequestBody(ref("OidcStartRequest")),
      responses: {
        "200": jsonResponse("Authorization URL created", ref("OidcStartResponse")),
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
          "Redirects back to the AutoPeer UI with oidc_state or oidc_error in the fragment",
        ),
      },
    },
  },
  "/auth/email/callback": {
    get: {
      tags: ["callbacks"],
      summary: "Handle the emailed registry auth callback",
      responses: {
        "302": redirectResponse(
          "Redirects back to the AutoPeer UI with email_token or email_error in the fragment",
        ),
      },
    },
  },
  "/v1/auth/oidc/complete": {
    post: {
      tags: ["auth"],
      summary: "Redeem a completed OIDC login state",
      requestBody: jsonRequestBody(ref("OidcCompleteRequest")),
      responses: {
        "200": jsonResponse("Session created", ref("AuthSessionResponse")),
        "400": errorResponse("OIDC state expired"),
        "401": errorResponse("Session expired"),
        "404": errorResponse("OIDC state or session was not found"),
        "409": errorResponse("OIDC login is still pending"),
      },
    },
  },
  "/v1/sessions": {
    get: {
      tags: ["sessions"],
      summary: "List node inventory and current sessions for the authenticated ASN",
      security: bearerSecurity(),
      responses: {
        "200": jsonResponse("Current session inventory", ref("SessionListResponse")),
        "401": errorResponse("Session token missing or expired"),
      },
    },
    post: {
      tags: ["sessions"],
      summary: "Create a new peering session request",
      security: bearerSecurity(),
      requestBody: jsonRequestBody(ref("CreateSessionRequest")),
      responses: {
        "200": jsonResponse("No change was necessary", ref("OperationStatus")),
        "202": jsonResponse("Operation created", ref("OperationStatus")),
        "400": errorResponse("Request payload is invalid"),
        "401": errorResponse("Session token missing or expired"),
        "403": errorResponse("Session is not allowed to mutate sessions"),
        "409": errorResponse("A session or pending operation already exists"),
        "502": errorResponse("Network repository state is unavailable"),
      },
    },
  },
  "/v1/sessions/{node}/{asn}": {
    patch: {
      tags: ["sessions"],
      summary: "Update an existing peering session",
      security: bearerSecurity(),
      parameters: [
        pathParam("node", "Node name"),
        pathParam("asn", "Authenticated ASN"),
      ],
      requestBody: jsonRequestBody(ref("UpdateSessionRequest")),
      responses: {
        "200": jsonResponse("No change was necessary", ref("OperationStatus")),
        "202": jsonResponse("Operation created", ref("OperationStatus")),
        "400": errorResponse("Request payload is invalid"),
        "401": errorResponse("Session token missing or expired"),
        "403": errorResponse("Path ASN does not match the current session"),
        "409": errorResponse("Session update conflicts with current state"),
        "502": errorResponse("Network repository state is unavailable"),
      },
    },
    delete: {
      tags: ["sessions"],
      summary: "Delete or retire a peering session",
      security: bearerSecurity(),
      parameters: [
        pathParam("node", "Node name"),
        pathParam("asn", "Authenticated ASN"),
      ],
      responses: {
        "200": jsonResponse("No change was necessary", ref("OperationStatus")),
        "202": jsonResponse("Operation created", ref("OperationStatus")),
        "401": errorResponse("Session token missing or expired"),
        "403": errorResponse("Path ASN does not match the current session"),
        "409": errorResponse("Session delete conflicts with current state"),
        "502": errorResponse("Network repository state is unavailable"),
      },
    },
  },
  "/v1/sessions/{node}/{asn}/migrate": {
    post: {
      tags: ["sessions"],
      summary: "Take over an existing session as a migration",
      security: bearerSecurity(),
      parameters: [
        pathParam("node", "Node name"),
        pathParam("asn", "Authenticated ASN"),
      ],
      responses: {
        "200": jsonResponse("No change was necessary", ref("OperationStatus")),
        "202": jsonResponse("Operation created", ref("OperationStatus")),
        "401": errorResponse("Session token missing or expired"),
        "403": errorResponse("Path ASN does not match the current session"),
        "409": errorResponse("Session migrate conflicts with current state"),
        "502": errorResponse("Network repository state is unavailable"),
      },
    },
  },
  "/v1/operations/{id}": {
    get: {
      tags: ["operations"],
      summary: "Refresh and read a previously created operation",
      security: bearerSecurity(),
      parameters: [
        pathParam("id", "Operation identifier"),
      ],
      responses: {
        "200": jsonResponse("Operation status", ref("OperationStatus")),
        "401": errorResponse("Session token missing or expired"),
        "404": errorResponse("Operation was not found for this ASN"),
      },
    },
  },
} satisfies JsonObject;

export function openApiSpec(request: Request, env: Env): JsonObject {
  return {
    openapi: "3.1.0",
    info: {
      title: "bird-lg-rs autopeer worker API",
      version: "0.1.0",
      description:
        "HTTP API for AutoPeer session auth, session management, and operation tracking. Protected endpoints use a bearer session_token returned by the auth flows.",
    },
    servers: [
      {
        url: apiServerUrl(request, env),
        description: "Configured API base URL",
      },
    ],
    tags: [
      { name: "meta", description: "Runtime config and health endpoints" },
      { name: "auth", description: "Authentication and session redemption flows" },
      { name: "callbacks", description: "Browser callback routes that redirect back to the UI" },
      { name: "sessions", description: "Session inventory and mutation endpoints" },
      { name: "operations", description: "Long-running operation status endpoints" },
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
    <title>AutoPeer API Docs</title>
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
