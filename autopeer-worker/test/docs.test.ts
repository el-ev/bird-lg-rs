import { describe, expect, it } from "vitest";

import worker from "../src/index";

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
    OIDC_PROVIDERS: "[]",
    HOST_ASNS: "4242421023",
    AUTOPEER_API_URL: "https://api.autopeer.example",
    AUTOPEER_SITE_URL: "https://autopeer.example",
    LOOKING_GLASS_URL: "https://lg.example",
    ...overrides,
  } as unknown as Env;
}

function runWorker(request: Request, env = makeEnv()): Promise<Response> {
  return worker.fetch(request as never, env);
}

describe("worker api docs", () => {
  it("serves an OpenAPI document for the public API routes", async () => {
    const response = await runWorker(new Request("https://worker.example/openapi.json"));

    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toContain("application/json");

    const body = await response.json() as Record<string, unknown>;
    expect(body.openapi).toBe("3.1.0");
    expect((body.info as Record<string, unknown>).title).toBe("bird-lg-rs autopeer worker API");

    const paths = body.paths as Record<string, unknown>;
    expect(paths["/v1/auth/start"]).toBeUndefined();
    expect(paths["/v1/auth/oidc/{provider}/start"]).toBeUndefined();
    expect(paths["/v1/auth/oidc/complete"]).toBeUndefined();
    expect(paths["/v1/sessions"]).toBeDefined();
    expect(paths["/v1/operations/{id}"]).toBeDefined();

    const components = body.components as Record<string, unknown>;
    const schemas = (components.schemas ?? {}) as Record<string, unknown>;
    expect(schemas.AuthStartRequest).toBeUndefined();

    const servers = body.servers as Array<Record<string, unknown>>;
    expect(servers[0]?.url).toBe("https://api.autopeer.example");
  });

  it("serves Swagger UI wired to the generated spec", async () => {
    const response = await runWorker(new Request("https://worker.example/swagger"));

    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toContain("text/html");

    const html = await response.text();
    expect(html).toContain("SwaggerUIBundle");
    expect(html).toContain('"/openapi.json"');
  });
});
