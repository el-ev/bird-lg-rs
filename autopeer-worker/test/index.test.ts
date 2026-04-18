import { describe, expect, it } from "vitest";

import {
  buildApplyWorkflowDispatchInputs,
  classifyMaintainerLookupError,
  decideApplyFailureRetry,
  decideNodeLockGate,
  decidePreMergeCheckGate,
  resolveDn42TargetPrefix,
} from "../src/index";

describe("peer-session-check merge gate", () => {
  it("waits when the validation workflow has not started yet", () => {
    expect(
      decidePreMergeCheckGate(
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
      decidePreMergeCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        undefined,
        Date.parse("2026-04-18T12:03:00.000Z"),
      ),
    ).toEqual({
      state: "failed",
      message: "peer-session-check did not start for your pull request.",
      shouldAttemptMerge: false,
    });
  });

  it("does not merge while peer-session-check is still running", () => {
    expect(
      decidePreMergeCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "in_progress", conclusion: null },
      ),
    ).toEqual({
      state: "pending_checks",
      message: "Your pull request is open; waiting for peer-session-check.",
      shouldAttemptMerge: false,
    });
  });

  it("allows merge only after peer-session-check completes successfully", () => {
    expect(
      decidePreMergeCheckGate(
        { created_at: "2026-04-18T12:00:00.000Z" },
        { status: "completed", conclusion: "success" },
      ),
    ).toEqual({
      state: "pending_merge",
      message: "Your checks passed; waiting for merge.",
      shouldAttemptMerge: true,
    });
  });
});

describe("node merge lock gate", () => {
  it("waits while another change still owns the node lock", () => {
    expect(decideNodeLockGate(false)).toEqual({
      state: "pending_merge",
      message: "Your checks passed; waiting for another change on this node to finish applying.",
      shouldAttemptMerge: false,
    });
  });

  it("allows merge once the node lock is free", () => {
    expect(decideNodeLockGate(true)).toEqual({
      state: "pending_merge",
      message: "Your checks passed; waiting for merge.",
      shouldAttemptMerge: true,
    });
  });
});

describe("apply retry gate", () => {
  const failedRun = {
    conclusion: "failure",
    created_at: "2026-04-18T12:00:00.000Z",
    updated_at: "2026-04-18T12:00:05.000Z",
  } as const;

  it("waits for the configured retry interval before dispatching", () => {
    expect(
      decideApplyFailureRetry(
        { apply_retry_count: 0 },
        failedRun,
        {
          retryLimit: 2,
          retryIntervalMs: 30_000,
        },
        Date.parse("2026-04-18T12:00:20.000Z"),
      ),
    ).toEqual({
      action: "wait",
      message: "peer-session-apply finished for your change with failure; retry 1/2 in 15s.",
    });
  });

  it("dispatches a retry once the interval has elapsed", () => {
    expect(
      decideApplyFailureRetry(
        { apply_retry_count: 0 },
        failedRun,
        {
          retryLimit: 2,
          retryIntervalMs: 30_000,
        },
        Date.parse("2026-04-18T12:00:40.000Z"),
      ),
    ).toEqual({
      action: "dispatch",
      attempt: 1,
      message: "peer-session-apply finished for your change with failure; starting retry 1/2.",
    });
  });

  it("fails once the configured retry limit is exhausted", () => {
    expect(
      decideApplyFailureRetry(
        { apply_retry_count: 2 },
        failedRun,
        {
          retryLimit: 2,
          retryIntervalMs: 30_000,
        },
      ),
    ).toEqual({
      action: "fail",
      message: "peer-session-apply finished for your change with failure after 2 retries.",
    });
  });
});

describe("apply workflow dispatch inputs", () => {
  it("targets the merged node and peer name with the default prefix", () => {
    expect(
      buildApplyWorkflowDispatchInputs({
        asn: "4242420454",
        node: "ams-01",
      }),
    ).toEqual({
      deploy_host: "ams-01",
      peer_targets: '["dn42_0454"]',
    });
  });

  it("respects a custom dn42 target prefix from common vars", () => {
    expect(
      buildApplyWorkflowDispatchInputs(
        {
          asn: "4242420454",
          node: "ams-01",
        },
        "peer_",
      ),
    ).toEqual({
      deploy_host: "ams-01",
      peer_targets: '["peer_0454"]',
    });
  });
});

describe("dn42 target prefix resolution", () => {
  it("reads dn42_prefix from common vars", () => {
    expect(resolveDn42TargetPrefix("dn42_prefix: peer_\n")).toBe("peer_");
  });

  it("falls back to the default prefix when common vars are absent or invalid", () => {
    expect(resolveDn42TargetPrefix(null)).toBe("dn42_");
    expect(resolveDn42TargetPrefix("[")).toBe("dn42_");
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
