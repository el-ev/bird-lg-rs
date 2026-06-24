import { describe, expect, it } from "vitest";

import worker, {
  decideApplyGate,
  decideCheckGate,
  decideNodeLockGate,
} from "../src/index";
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

describe("API endpoint error i18n", () => {
  const env = {} as Env;

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
