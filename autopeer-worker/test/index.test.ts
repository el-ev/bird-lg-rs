import { describe, expect, it } from "vitest";

import worker, {
  classifyMaintainerLookupError,
  decideApplyGate,
  decideCheckGate,
  decideNodeLockGate,
  resolveEffectiveMaintainer,
} from "../src/index";
import { NoMaintainerError, RegistryPathNotFoundError } from "../src/registry";
import { uiMessage } from "../src/utils";

describe("peer-session-check gate", () => {
  it("waits when the validation workflow has not started yet", () => {
    expect(
      decideCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        undefined,
        Date.parse("2026-04-18T12:00:30.000Z"),
      ),
    ).toEqual({
      state: "pending_checks",
      message: uiMessage("operation.message.check_wait_start"),
      shouldAttemptMerge: false,
    });
  });

  it("fails closed when peer-session-check never appears", () => {
    expect(
      decideCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        undefined,
        Date.parse("2026-04-18T12:06:00.000Z"),
      ),
    ).toEqual({
      state: "failed",
      message: uiMessage("operation.message.check_not_started"),
      shouldAttemptMerge: false,
    });
  });

  it("does not advance while peer-session-check is still running", () => {
    expect(
      decideCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "in_progress", conclusion: null },
      ),
    ).toEqual({
      state: "pending_checks",
      message: uiMessage("operation.message.pending_checks"),
      shouldAttemptMerge: false,
    });
  });

  it("advances to applying after peer-session-check completes successfully", () => {
    expect(
      decideCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "completed", conclusion: "success" },
      ),
    ).toEqual({
      state: "applying",
      message: uiMessage("operation.message.applying"),
      shouldAttemptMerge: false,
    });
  });

  it("marks failure when peer-session-check concludes with failure", () => {
    expect(
      decideCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "completed", conclusion: "failure" },
      ),
    ).toEqual({
      state: "failed",
      message: uiMessage("operation.message.check_failed", { conclusion: "failure" }),
      shouldAttemptMerge: false,
    });
  });
});

describe("peer-session-apply gate (PR mode)", () => {
  it("waits while apply has not started yet", () => {
    expect(
      decideApplyGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        undefined,
        Date.parse("2026-04-18T12:01:00.000Z"),
      ),
    ).toEqual({
      state: "applying",
      message: uiMessage("operation.message.apply_wait_start"),
      shouldAttemptMerge: false,
    });
  });

  it("does not advance while apply is still running (preflight or deploy)", () => {
    expect(
      decideApplyGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "in_progress", conclusion: null },
      ),
    ).toEqual({
      state: "applying",
      message: uiMessage("operation.message.applying"),
      shouldAttemptMerge: false,
    });
  });

  it("allows merge only after peer-session-apply completes successfully on the PR", () => {
    expect(
      decideApplyGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "completed", conclusion: "success" },
      ),
    ).toEqual({
      state: "pending_merge",
      message: uiMessage("operation.message.pending_merge"),
      shouldAttemptMerge: true,
    });
  });

  it("marks failure when peer-session-apply concludes with failure (e.g. preflight rejected unreachable node)", () => {
    expect(
      decideApplyGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "completed", conclusion: "failure" },
      ),
    ).toEqual({
      state: "failed",
      message: uiMessage("operation.message.apply_failed", { conclusion: "failure" }),
      shouldAttemptMerge: false,
    });
  });
});

describe("node merge lock gate", () => {
  it("waits while another change still owns the node lock", () => {
    expect(decideNodeLockGate(false)).toEqual({
      state: "pending_merge",
      message: uiMessage("operation.message.wait_node_lock"),
      shouldAttemptMerge: false,
    });
  });

  it("allows merge once the node lock is free", () => {
    expect(decideNodeLockGate(true)).toEqual({
      state: "pending_merge",
      message: uiMessage("operation.message.pending_merge"),
      shouldAttemptMerge: true,
    });
  });
});

describe("ASN lookup error classification", () => {
  it("marks missing aut-num objects as invalid ASNs", () => {
    const error = classifyMaintainerLookupError(
      "4242429999",
      new RegistryPathNotFoundError("data/aut-num/AS4242429999"),
    );

    expect(error.status).toBe(400);
    expect(error.uiMessage).toEqual(uiMessage("error.auth.asn.not_found", { asn: "4242429999" }));
  });

  it("keeps non-missing registry issues out of the invalid-ASN bucket", () => {
    const error = classifyMaintainerLookupError(
      "4242421024",
      new NoMaintainerError("4242421024"),
    );

    expect(error.status).toBe(400);
    expect(error.uiMessage).toEqual(
      uiMessage("error.auth.asn.no_supported_auth", { asn: "4242421024" }),
    );
  });
});

describe("host impersonation maintainer resolution", () => {
  const maintainer = (name: string) => ({
    name,
    auth_lines: [],
    ssh_public_keys: [],
    ssh_fingerprints: [],
    pgp_fingerprints: [],
    contact_emails: [],
  });

  it("lists available mntners when effective_mnt is missing for a multi-mntner ASN", () => {
    expect(() =>
      resolveEffectiveMaintainer([maintainer("ROUTEDBITS-MNT"), maintainer("IRIS-MNT")]),
    ).toThrowError("error.auth.impersonation.maintainer.required");
  });

  it("lists available mntners when the requested maintainer is not present", () => {
    expect(() =>
      resolveEffectiveMaintainer(
        [maintainer("ROUTEDBITS-MNT"), maintainer("IRIS-MNT")],
        "OTHER-MNT",
      ),
    ).toThrowError("error.auth.impersonation.maintainer.missing");
  });
});

describe("API endpoint error i18n", () => {
  const env = {} as Env;

  function jsonRequest(path: string, body: unknown): Request {
    return new Request(`https://autopeer.example${path}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
  }

  it("returns uiMessage key for a missing required field on /v1/auth/start", async () => {
    const response = await worker.fetch(jsonRequest("/v1/auth/start", {}) as never, env);
    expect(response.status).toBe(400);
    const body = (await response.json()) as { error: { key: string; params?: Record<string, string> } };
    expect(body.error).toEqual({
      key: "error.field.required",
      params: { field: "asn" },
    });
  });

  it("returns uiMessage key for a malformed ASN", async () => {
    const response = await worker.fetch(
      jsonRequest("/v1/auth/start", { asn: "not-an-asn" }) as never,
      env,
    );
    expect(response.status).toBe(400);
    const body = (await response.json()) as { error: { key: string } };
    expect(body.error.key).toBe("error.auth.asn.unsupported");
  });

  it("returns uiMessage key when request body is not valid JSON", async () => {
    const response = await worker.fetch(
      new Request("https://autopeer.example/v1/auth/start", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: "{",
      }) as never,
      env,
    );
    expect(response.status).toBe(400);
    const body = (await response.json()) as { error: { key: string } };
    expect(body.error.key).toBe("error.request.body.invalid_json");
  });

  it("returns uiMessage key with bearer token missing", async () => {
    const response = await worker.fetch(
      new Request("https://autopeer.example/v1/sessions", {
        method: "GET",
      }) as never,
      env,
    );
    expect(response.status).toBe(401);
    const body = (await response.json()) as { error: { key: string } };
    expect(body.error.key).toBe("error.auth.session.token.missing");
  });
});
