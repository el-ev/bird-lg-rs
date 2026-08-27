import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  RegistryPathNotFoundError,
  RegistryUnavailableError,
  diagnoseRegistryAccess,
  loadMaintainersForAsn,
} from "./registry";

const env = {
  DN42_REGISTRY_BASE_URL: "https://git.example.test",
  DN42_REGISTRY_OWNER: "dn42",
  DN42_REGISTRY_REPO: "registry",
  DN42_REGISTRY_BRANCH: "master",
  DN42_GIT_TOKEN: "test-token",
};

const REPO_API = "/api/v1/repos/dn42/registry";
const AUT_NUM_API = `${REPO_API}/contents/data/aut-num/AS4242421024`;

type Route = { status: number; body?: unknown };

function contentBody(text: string): Route {
  return { status: 200, body: { content: btoa(text), encoding: "base64" } };
}

function stubFetch(routes: Record<string, Route>) {
  const mock = vi.fn(async (input: RequestInfo | URL) => {
    const url = new URL(input instanceof Request ? input.url : String(input));
    const route = routes[url.pathname];
    if (!route) throw new Error(`unexpected fetch: ${url.pathname}`);
    return new Response(route.body === undefined ? null : JSON.stringify(route.body), {
      status: route.status,
    });
  });
  vi.stubGlobal("fetch", mock);
  return mock;
}

async function caughtLookupError(): Promise<unknown> {
  return loadMaintainersForAsn(env, "4242421024").then(
    () => {
      throw new Error("expected loadMaintainersForAsn to reject");
    },
    (error: unknown) => error,
  );
}

describe("loadMaintainersForAsn failure classification", () => {
  beforeEach(() => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("reports an unknown ASN only after the registry itself checks out", async () => {
    const mock = stubFetch({
      [AUT_NUM_API]: { status: 404 },
      [REPO_API]: { status: 200, body: {} },
      [`${REPO_API}/branches/master`]: { status: 200, body: {} },
    });

    const error = await caughtLookupError();
    expect(error).toBeInstanceOf(RegistryPathNotFoundError);
    expect(mock).toHaveBeenCalledTimes(3);
  });

  it("classifies a hidden repository as registry unavailable, not unknown ASN", async () => {
    stubFetch({
      [AUT_NUM_API]: { status: 404 },
      [REPO_API]: { status: 404 },
    });

    const error = await caughtLookupError();
    expect(error).toBeInstanceOf(RegistryUnavailableError);
    expect((error as RegistryUnavailableError).reason).toBe("repo_not_visible");
    expect(console.error).toHaveBeenCalled();
  });

  it("classifies a missing branch as registry unavailable", async () => {
    stubFetch({
      [AUT_NUM_API]: { status: 404 },
      [REPO_API]: { status: 200, body: {} },
      [`${REPO_API}/branches/master`]: { status: 404 },
    });

    const error = await caughtLookupError();
    expect(error).toBeInstanceOf(RegistryUnavailableError);
    expect((error as RegistryUnavailableError).reason).toBe("branch_missing");
  });

  it("classifies a rejected token without probing further", async () => {
    const mock = stubFetch({
      [AUT_NUM_API]: { status: 401 },
    });

    const error = await caughtLookupError();
    expect(error).toBeInstanceOf(RegistryUnavailableError);
    expect((error as RegistryUnavailableError).reason).toBe("token_rejected");
    expect(mock).toHaveBeenCalledTimes(1);
  });

  it("classifies a forbidden request as registry unavailable", async () => {
    stubFetch({
      [AUT_NUM_API]: { status: 403 },
    });

    const error = await caughtLookupError();
    expect(error).toBeInstanceOf(RegistryUnavailableError);
    expect((error as RegistryUnavailableError).reason).toBe("access_forbidden");
  });

  it("flags a missing mntner as data inconsistency, keeping its path", async () => {
    stubFetch({
      [AUT_NUM_API]: contentBody("aut-num: AS4242421024\nmnt-by: GONE-MNT"),
      [`${REPO_API}/contents/data/mntner/GONE-MNT`]: { status: 404 },
    });

    const error = await caughtLookupError();
    expect(error).toBeInstanceOf(RegistryPathNotFoundError);
    expect((error as RegistryPathNotFoundError).path).toBe("data/mntner/GONE-MNT");
    expect(console.warn).toHaveBeenCalled();
  });

  it("keeps optional person/role 404s cheap and loads maintainers", async () => {
    const mock = stubFetch({
      [AUT_NUM_API]: contentBody("aut-num: AS4242421024\nmnt-by: IRIS-MNT"),
      [`${REPO_API}/contents/data/mntner/IRIS-MNT`]: contentBody(
        ["mntner: IRIS-MNT", `auth: ssh-ed25519 ${btoa("test-key-blob")}`, "admin-c: IRIS-DN42"].join("\n"),
      ),
      [`${REPO_API}/contents/data/person/IRIS-DN42`]: { status: 404 },
      [`${REPO_API}/contents/data/role/IRIS-DN42`]: contentBody("role: Iris\ne-mail: iris@example.test"),
    });

    const maintainers = await loadMaintainersForAsn(env, "4242421024");
    expect(maintainers).toHaveLength(1);
    expect(maintainers[0]?.ssh_public_keys).toHaveLength(1);
    expect(maintainers[0]?.contact_emails).toEqual(["iris@example.test"]);
    const probed = mock.mock.calls.some((call) => {
      const url = new URL(call[0] instanceof Request ? call[0].url : String(call[0]));
      return url.pathname === REPO_API;
    });
    expect(probed).toBe(false);
  });
});

describe("diagnoseRegistryAccess", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("returns null when repo and branch are readable", async () => {
    stubFetch({
      [REPO_API]: { status: 200, body: {} },
      [`${REPO_API}/branches/master`]: { status: 200, body: {} },
    });

    await expect(diagnoseRegistryAccess(env)).resolves.toBeNull();
  });

  it("reports network failures as request_failed with status 0", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => {
      throw new TypeError("network down");
    }));

    await expect(diagnoseRegistryAccess(env)).resolves.toEqual({
      reason: "request_failed",
      status: 0,
    });
  });
});
