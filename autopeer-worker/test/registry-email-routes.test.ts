import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  ChallengeRecord,
  RegistryEmailAuthRequestRecord,
  SessionRecord,
} from "../src/types";

const dbMocks = vi.hoisted(() => ({
  getChallenge: vi.fn(),
  putRegistryEmailAuthRequest: vi.fn(),
  getRegistryEmailAuthRequest: vi.fn(),
  getRegistryEmailAuthRequestByToken: vi.fn(),
  deleteRegistryEmailAuthRequest: vi.fn(),
  putAuthSession: vi.fn(),
  getAuthSession: vi.fn(),
}));

const mailerMocks = vi.hoisted(() => ({
  sendRegistryEmailAuthMessage: vi.fn(),
}));

vi.mock("../src/db", async () => {
  const actual = await vi.importActual<typeof import("../src/db")>("../src/db");
  return {
    ...actual,
    getChallenge: dbMocks.getChallenge,
    putRegistryEmailAuthRequest: dbMocks.putRegistryEmailAuthRequest,
    getRegistryEmailAuthRequest: dbMocks.getRegistryEmailAuthRequest,
    getRegistryEmailAuthRequestByToken: dbMocks.getRegistryEmailAuthRequestByToken,
    deleteRegistryEmailAuthRequest: dbMocks.deleteRegistryEmailAuthRequest,
    putAuthSession: dbMocks.putAuthSession,
    getAuthSession: dbMocks.getAuthSession,
  };
});

vi.mock("../src/mailer", async () => {
  const actual = await vi.importActual<typeof import("../src/mailer")>("../src/mailer");
  return {
    ...actual,
    sendRegistryEmailAuthMessage: mailerMocks.sendRegistryEmailAuthMessage,
  };
});

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
    AUTOPEER_URL: "https://api.autopeer.example",
    AUTOPEER_SITE_URL: "https://autopeer.example",
    LOOKING_GLASS_URL: "https://lg.example",
    ...overrides,
  } as unknown as Env;
}

function jsonRequest(path: string, body: unknown): Request {
  return new Request(`https://worker.example${path}`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
}

function proxiedGetRequest(path: string): Request {
  return new Request(`https://worker.example${path}`, {
    headers: {
      "x-forwarded-host": "autopeer.iris.dn42",
      "x-forwarded-proto": "https",
    },
  });
}

function runWorker(request: Request, env = makeEnv()): Promise<Response> {
  return worker.fetch(request as never, env);
}

function challengeRecord(): ChallengeRecord {
  return {
    id: "challenge-1",
    asn: "4242421024",
    challenge_text: [
      "dn42-autopeer challenge",
      "asn: 4242421024",
      "challenge_id: challenge-1",
      "issued_at: 2026-04-19T01:23:45.000Z",
    ].join("\n"),
    methods: [
      {
        kind: "registry_email",
        label: "Registry Email Magic Link",
        description: "Email auth",
        email_targets: [
          {
            maintainer: "EXAMPLE-MNT",
            emails: ["admin@example.net"],
          },
          {
            maintainer: "SECOND-MNT",
            emails: ["ops@example.net"],
          },
        ],
      },
    ],
    maintainers: [
      {
        name: "EXAMPLE-MNT",
        auth_lines: [],
        ssh_public_keys: [],
        ssh_fingerprints: [],
        pgp_fingerprints: [],
        contact_emails: ["admin@example.net"],
      },
      {
        name: "SECOND-MNT",
        auth_lines: [],
        ssh_public_keys: [],
        ssh_fingerprints: [],
        pgp_fingerprints: [],
        contact_emails: ["ops@example.net"],
      },
    ],
    created_at: "2026-04-19T01:23:45.000Z",
    expires_at: "2999-01-01T00:00:00.000Z",
  };
}

function emailAuthRequest(
  overrides: Partial<RegistryEmailAuthRequestRecord> = {},
): RegistryEmailAuthRequestRecord {
  return {
    challenge_id: "challenge-1",
    effective_mnt: "EXAMPLE-MNT",
    email_snapshot: ["admin@example.net"],
    code: "12345678",
    token: "email-token-1",
    session_token: null,
    created_at: "2026-04-19T01:23:45.000Z",
    expires_at: "2999-01-01T00:00:00.000Z",
    ...overrides,
  };
}

function sessionRecord(overrides: Partial<SessionRecord> = {}): SessionRecord {
  return {
    token: "session-1",
    asn: "4242421024",
    effective_mnt: "EXAMPLE-MNT",
    auth_method: {
      kind: "registry_email",
      label: "Registry Email Magic Link",
      description: "You authenticated with EXAMPLE-MNT using registry email auth.",
    },
    created_at: "2026-04-19T01:23:45.000Z",
    expires_at: "2999-01-01T06:00:00.000Z",
    ...overrides,
  };
}

describe("registry email worker routes", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mailerMocks.sendRegistryEmailAuthMessage.mockResolvedValue(undefined);
  });

  it("sends registry email auth to the selected maintainer contacts", async () => {
    dbMocks.getChallenge.mockResolvedValue(challengeRecord());

    const response = await runWorker(
      jsonRequest("/v1/auth/verify/registry-email/send", {
        challenge_id: "challenge-1",
        effective_mnt: "SECOND-MNT",
      }),
    );

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual({
      effective_mnt: "SECOND-MNT",
      emails: ["ops@example.net"],
      expires_at: expect.any(String),
    });
    expect(mailerMocks.sendRegistryEmailAuthMessage).toHaveBeenCalled();
    expect(dbMocks.putRegistryEmailAuthRequest).toHaveBeenCalledWith(
      expect.anything(),
      expect.objectContaining({
        challenge_id: "challenge-1",
        effective_mnt: "SECOND-MNT",
        email_snapshot: ["ops@example.net"],
      }),
    );
  });

  it("redirects a completed magic link callback through the forwarded host", async () => {
    dbMocks.getRegistryEmailAuthRequestByToken.mockResolvedValue(
      emailAuthRequest({ session_token: "session-1" }),
    );

    const response = await runWorker(
      proxiedGetRequest("/auth/email/callback?challenge_id=challenge-1&token=email-token-1"),
    );

    expect(response.status).toBe(302);
    expect(response.headers.get("location")).toBe(
      "https://autopeer.iris.dn42/#email_token=email-token-1",
    );
  });

  it("completes a registry email login from the magic-link token", async () => {
    dbMocks.getRegistryEmailAuthRequestByToken.mockResolvedValue(
      emailAuthRequest({ session_token: "session-1" }),
    );
    dbMocks.getAuthSession.mockResolvedValue(sessionRecord());

    const response = await runWorker(
      jsonRequest("/v1/auth/verify/registry-email/complete", {
        token: "email-token-1",
      }),
    );

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toMatchObject({
      session_token: "session-1",
      asn: "4242421024",
      effective_mnt: "EXAMPLE-MNT",
      auth_method: {
        kind: "registry_email",
      },
    });
  });
});
