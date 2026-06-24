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
  return readOptionalEnvString(env, "AUTOPEER_API_URL") ?? new URL(request.url).origin;
}

const components = {
  securitySchemes: {
    bearerAuth: {
      type: "http",
      scheme: "bearer",
      bearerFormat: "session_token",
      description: "Use the session_token returned by the configured auth_url.",
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
    RuntimeConfigResponse: {
      type: "object",
      required: ["autopeer_api_url", "autopeer_site_url", "auth_url", "oidc_methods"],
      properties: {
        autopeer_api_url: { type: "string", format: "uri" },
        autopeer_site_url: { type: "string", format: "uri" },
        looking_glass_url: { type: "string", format: "uri" },
        auth_url: {
          type: "string",
          format: "uri",
          description: "Central auth site used to obtain bearer session tokens.",
        },
        oidc_methods: {
          type: "array",
          items: { type: "object" },
          deprecated: true,
          description: "Kept for frontend compatibility; OIDC options are now loaded from auth_url.",
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
        mp_bgp_transport: {
          type: "string",
          enum: ["ipv4", "ipv6"],
        },
        peering_strategy: {
          type: "string",
          enum: ["full_table", "transit", "peer", "downstream"],
        },
        psk: {
          type: "string",
          description: "WireGuard pre-shared key (write-only, 44-char base64)",
        },
        has_psk: {
          type: "boolean",
          readOnly: true,
          description: "Whether a WireGuard PSK is configured (read-only)",
        },
        encrypt_endpoint: {
          type: "boolean",
          description: "Encrypt the endpoint field with Ansible Vault in the git repo",
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
          enum: ["managed", "manual", "locked", "pending_pr", "stalled_pr", "conflict"],
        },
        spec: ref("PeerSessionSpec"),
        metadata: ref("SessionMetadata"),
        has_psk: { type: "boolean", description: "Whether a WireGuard PSK is configured" },
        has_encrypted_endpoint: { type: "boolean", description: "Whether the endpoint is vault-encrypted" },
        pending_operation_id: { type: "string" },
        pull_request_url: { type: "string", format: "uri" },
        message: ref("UiMessage"),
      },
    },
    SessionListResponse: {
      type: "object",
      required: ["asn", "nodes", "sessions"],
      properties: {
        asn: { type: "string" },
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
          enum: ["create", "update", "retire", "delete", "migrate"],
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
      summary: "Permanently delete a peering session",
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
  "/v1/sessions/{node}/{asn}/retire": {
    post: {
      tags: ["sessions"],
      summary: "Retire (disable) a peering session",
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
        "409": errorResponse("Session retire conflicts with current state"),
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
  "/v1/operations/{id}/retry": {
    post: {
      tags: ["operations"],
      summary: "Retry a failed operation by rebasing and force-pushing its branch",
      security: bearerSecurity(),
      parameters: [
        pathParam("id", "Operation identifier"),
      ],
      responses: {
        "202": jsonResponse("Operation retried", ref("OperationStatus")),
        "401": errorResponse("Session token missing or expired"),
        "404": errorResponse("Operation was not found for this ASN"),
        "409": errorResponse("Operation is not in a retryable state"),
        "502": errorResponse("Branch content is unavailable"),
      },
    },
  },
  "/v1/operations/{id}/drop": {
    post: {
      tags: ["operations"],
      summary: "Drop a failed operation, closing its PR and deleting the branch",
      security: bearerSecurity(),
      parameters: [
        pathParam("id", "Operation identifier"),
      ],
      responses: {
        "200": jsonResponse("Operation dropped", ref("OperationStatus")),
        "401": errorResponse("Session token missing or expired"),
        "404": errorResponse("Operation was not found for this ASN"),
        "409": errorResponse("Operation is not in a droppable state"),
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
        "HTTP API for AutoPeer session management and operation tracking. Authentication is handled by the configured auth_url; protected endpoints use the returned bearer session_token.",
    },
    servers: [
      {
        url: apiServerUrl(request, env),
        description: "Configured AutoPeer API base URL",
      },
    ],
    tags: [
      { name: "meta", description: "Runtime config and health endpoints" },
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
