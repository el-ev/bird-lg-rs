import { describe, expect, it } from "vitest";

import {
  classifyMaintainerLookupError,
  decideApplyGate,
  decideCheckGate,
  decideNodeLockGate,
} from "../src/index";

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
      message: "Your pull request is open; waiting for peer-session-check to start.",
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
      message: "peer-session-check did not start for your pull request.",
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
      message: "Your pull request is open; waiting for peer-session-check.",
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
      message: "Checks passed; applying your session to the node for verification.",
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
      message: "peer-session-check finished with failure",
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
      message: "Checks passed; waiting for peer-session-apply to start.",
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
      message: "Checks passed; applying your session to the node for verification.",
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
      message: "Apply succeeded on the node; waiting for merge.",
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
      message: "peer-session-apply finished with failure",
      shouldAttemptMerge: false,
    });
  });
});

describe("node merge lock gate", () => {
  it("waits while another change still owns the node lock", () => {
    expect(decideNodeLockGate(false)).toEqual({
      state: "pending_merge",
      message: "Apply succeeded; waiting for another change on this node to finish merging.",
      shouldAttemptMerge: false,
    });
  });

  it("allows merge once the node lock is free", () => {
    expect(decideNodeLockGate(true)).toEqual({
      state: "pending_merge",
      message: "Apply succeeded on the node; waiting for merge.",
      shouldAttemptMerge: true,
    });
  });
});

describe("ASN lookup error classification", () => {
  it("marks missing aut-num objects as invalid ASNs", () => {
    const error = classifyMaintainerLookupError(
      "4242429999",
      new Error("Registry path not found: data/aut-num/AS4242429999"),
    );

    expect(error.status).toBe(400);
    expect(error.message).toBe(
      "AS4242429999 is invalid because it does not exist in the DN42 registry.",
    );
  });

  it("keeps non-missing registry issues out of the invalid-ASN bucket", () => {
    const error = classifyMaintainerLookupError(
      "4242421024",
      new Error("AS4242421024 does not expose any mnt-by records in the registry"),
    );

    expect(error.status).toBe(400);
    expect(error.message).toBe(
      "AS4242421024 exists in DN42, but it does not publish maintainer auth we can use yet.",
    );
  });
});
