import { createHash } from "node:crypto";

import type { AuthMethod, MaintainerRecord, RegistryEmailTarget } from "./types";
import { fromBase64, joinPath, readSecret, uiMessage } from "./utils";

export class RegistryPathNotFoundError extends Error {
  constructor(public readonly path: string) {
    super(`Registry path not found: ${path}`);
  }
}

export type RegistryAccessReason =
  | "token_rejected"
  | "access_forbidden"
  | "repo_not_visible"
  | "branch_missing"
  | "request_failed"
  | "invalid_payload";

const REGISTRY_ACCESS_HINTS: Record<RegistryAccessReason, string> = {
  token_rejected:
    "DN42_GIT_TOKEN was rejected: the token is invalid, expired, or revoked. Generate a new token and redeploy the secret.",
  access_forbidden:
    "the registry refused the request: the token may lack the read:repository scope, or the instance requires sign-in",
  repo_not_visible:
    "the token authenticates but cannot see the registry repository: check for a public-only token, a restricted account, or a moved/renamed repo",
  branch_missing: "the registry repository is visible but the configured branch does not exist",
  request_failed: "the registry API returned an unexpected status",
  invalid_payload: "the registry API returned an unexpected response body",
};

export class RegistryUnavailableError extends Error {
  readonly hint: string;

  constructor(
    public readonly reason: RegistryAccessReason,
    public readonly path: string,
    public readonly status: number,
  ) {
    const hint = REGISTRY_ACCESS_HINTS[reason];
    super(`Registry unavailable (${reason}, HTTP ${status}) for ${path}: ${hint}`);
    this.hint = hint;
  }
}

function registryUnavailable(
  env: RegistryEnv,
  reason: RegistryAccessReason,
  path: string,
  status: number,
): RegistryUnavailableError {
  const error = new RegistryUnavailableError(reason, path, status);
  console.error("registry access failure", {
    reason,
    path,
    status,
    registry: `${env.DN42_REGISTRY_BASE_URL} ${env.DN42_REGISTRY_OWNER}/${env.DN42_REGISTRY_REPO}@${env.DN42_REGISTRY_BRANCH}`,
    hint: error.hint,
  });
  return error;
}

export class NoMaintainerError extends Error {
  constructor(public readonly asn: string) {
    super(`AS${asn} does not expose any mnt-by records in the registry`);
  }
}

interface GiteaContentResponse {
  content?: string;
  encoding?: string;
}

export interface RegistryEnv {
  DN42_REGISTRY_BASE_URL: string;
  DN42_REGISTRY_OWNER: string;
  DN42_REGISTRY_REPO: string;
  DN42_REGISTRY_BRANCH: string;
}

function parseFieldValues(text: string, field: string): string[] {
  const prefix = `${field.toLowerCase()}:`;
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.toLowerCase().startsWith(prefix))
    .map((line) => line.slice(prefix.length).trim())
    .filter(Boolean);
}

export function sshPublicKeyFingerprint(publicKey: string): string | null {
  const [, keyBlob] = publicKey.trim().split(/\s+/, 3);
  if (!keyBlob || !/^[A-Za-z0-9+/]+={0,2}$/.test(keyBlob) || keyBlob.length % 4 === 1) {
    return null;
  }

  const rawKey = Buffer.from(keyBlob, "base64");
  if (rawKey.length === 0) {
    return null;
  }

  const digest = createHash("sha256").update(rawKey).digest("base64").replace(/=+$/u, "");
  return `SHA256:${digest}`;
}

function parseMaintainer(name: string, text: string): MaintainerRecord {
  const authLines = parseFieldValues(text, "auth");
  const sshPublicKeys = authLines.filter((line) => line.startsWith("ssh-"));
  return {
    name,
    auth_lines: authLines,
    ssh_public_keys: sshPublicKeys,
    ssh_fingerprints: [
      ...new Set(
        sshPublicKeys
          .map((key) => sshPublicKeyFingerprint(key))
          .filter((fingerprint): fingerprint is string => fingerprint !== null),
      ),
    ],
    pgp_fingerprints: authLines
      .filter((line) => line.toLowerCase().startsWith("pgp-fingerprint"))
      .map((line) => line.split(/\s+/).slice(1).join("").toUpperCase()),
    contact_emails: [],
  };
}

async function fetchRegistryContent(env: RegistryEnv, path: string): Promise<string> {
  const url = new URL(
    `/api/v1/repos/${joinPath(env.DN42_REGISTRY_OWNER, env.DN42_REGISTRY_REPO)}/contents/${joinPath(path)}`,
    env.DN42_REGISTRY_BASE_URL,
  );
  url.searchParams.set("ref", env.DN42_REGISTRY_BRANCH);

  const response = await fetch(url, {
    headers: {
      accept: "application/json",
      authorization: `token ${readSecret(env, "DN42_GIT_TOKEN")}`,
    },
  });

  if (response.status === 404) {
    throw new RegistryPathNotFoundError(path);
  }
  if (response.status === 401) {
    throw registryUnavailable(env, "token_rejected", path, response.status);
  }
  if (response.status === 403) {
    throw registryUnavailable(env, "access_forbidden", path, response.status);
  }
  if (!response.ok) {
    throw registryUnavailable(env, "request_failed", path, response.status);
  }

  const body = (await response.json()) as GiteaContentResponse;
  if (body.encoding !== "base64" || typeof body.content !== "string") {
    throw registryUnavailable(env, "invalid_payload", path, response.status);
  }

  return fromBase64(body.content);
}

async function registryApiStatus(env: RegistryEnv, apiPath: string): Promise<number> {
  try {
    const response = await fetch(new URL(apiPath, env.DN42_REGISTRY_BASE_URL), {
      headers: {
        accept: "application/json",
        authorization: `token ${readSecret(env, "DN42_GIT_TOKEN")}`,
      },
    });
    return response.status;
  } catch {
    return 0;
  }
}

export interface RegistryAccessProblem {
  reason: RegistryAccessReason;
  status: number;
}

export async function diagnoseRegistryAccess(env: RegistryEnv): Promise<RegistryAccessProblem | null> {
  const repoPath = `/api/v1/repos/${joinPath(env.DN42_REGISTRY_OWNER, env.DN42_REGISTRY_REPO)}`;
  const repoStatus = await registryApiStatus(env, repoPath);
  if (repoStatus === 401) return { reason: "token_rejected", status: repoStatus };
  if (repoStatus === 403) return { reason: "access_forbidden", status: repoStatus };
  if (repoStatus === 404) return { reason: "repo_not_visible", status: repoStatus };
  if (repoStatus !== 200) return { reason: "request_failed", status: repoStatus };

  const branchStatus = await registryApiStatus(
    env,
    `${repoPath}/branches/${joinPath(env.DN42_REGISTRY_BRANCH)}`,
  );
  if (branchStatus === 404) return { reason: "branch_missing", status: branchStatus };
  if (branchStatus !== 200) return { reason: "request_failed", status: branchStatus };
  return null;
}

async function fetchOptionalRegistryContent(env: RegistryEnv, path: string): Promise<string | null> {
  try {
    return await fetchRegistryContent(env, path);
  } catch (error) {
    if (error instanceof RegistryPathNotFoundError && error.path === path) {
      return null;
    }
    throw error;
  }
}

async function loadContactEmailsForHandle(env: RegistryEnv, handle: string): Promise<string[]> {
  const personText = await fetchOptionalRegistryContent(env, `data/person/${handle}`);
  const roleText = personText === null
    ? await fetchOptionalRegistryContent(env, `data/role/${handle}`)
    : null;
  const text = personText ?? roleText;
  if (text === null) {
    return [];
  }
  return [...new Set(parseFieldValues(text, "e-mail"))];
}

export async function loadMaintainersForAsn(env: RegistryEnv, asn: string): Promise<MaintainerRecord[]> {
  const autNumPath = `data/aut-num/AS${asn}`;
  let autNumText: string;
  try {
    autNumText = await fetchRegistryContent(env, autNumPath);
  } catch (error) {
    // A 404 here means either the ASN genuinely does not exist or the whole
    // registry is invisible to our token (Gitea hides repos from public-only
    // tokens and restricted accounts with 404 too). Only report the ASN as
    // unknown once the repo and branch themselves check out.
    if (error instanceof RegistryPathNotFoundError) {
      const problem = await diagnoseRegistryAccess(env);
      if (problem) {
        throw registryUnavailable(env, problem.reason, autNumPath, problem.status);
      }
    }
    throw error;
  }
  const maintainerNames = [...new Set(parseFieldValues(autNumText, "mnt-by"))];

  if (maintainerNames.length === 0) {
    throw new NoMaintainerError(asn);
  }

  const maintainers = await Promise.all(
    maintainerNames.map(async (name) => {
      let text: string;
      try {
        text = await fetchRegistryContent(env, `data/mntner/${name}`);
      } catch (error) {
        if (error instanceof RegistryPathNotFoundError) {
          console.warn("registry data inconsistency", {
            asn,
            missing_mntner: name,
            referenced_by: autNumPath,
          });
        }
        throw error;
      }
      const maintainer = parseMaintainer(name, text);
      const contactHandles = [
        ...new Set([
          ...parseFieldValues(text, "admin-c"),
          ...parseFieldValues(text, "tech-c"),
        ]),
      ];
      const contactEmails = await Promise.all(
        contactHandles.map(async (handle) => loadContactEmailsForHandle(env, handle)),
      );
      maintainer.contact_emails = [...new Set(contactEmails.flat())];
      return maintainer;
    }),
  );

  return maintainers;
}

export function methodsFromMaintainers(
  maintainers: MaintainerRecord[],
  oidcMethods: AuthMethod[],
  options: {
    registryEmailEnabled?: boolean;
  } = {},
): AuthMethod[] {
  const methods: AuthMethod[] = [];
  const { registryEmailEnabled = true } = options;
  const sshFingerprints = [
    ...new Set(maintainers.flatMap((mnt) => mnt.ssh_fingerprints)),
  ];
  const pgpFingerprints = [
    ...new Set(maintainers.flatMap((mnt) => mnt.pgp_fingerprints)),
  ];
  const emailTargets: RegistryEmailTarget[] = maintainers
    .map((maintainer) => ({
      maintainer: maintainer.name,
      emails: [...new Set(maintainer.contact_emails)],
    }))
    .filter((target) => target.emails.length > 0);

  if (maintainers.some((mnt) => mnt.ssh_public_keys.length > 0)) {
    methods.push({
      kind: "registry_ssh",
      label: uiMessage("auth_method.registry_ssh.label"),
      description: uiMessage("auth_method.registry_ssh.description"),
      ssh_fingerprints: sshFingerprints,
      pgp_fingerprints: [],
      email_targets: [],
    });
  }

  if (pgpFingerprints.length > 0) {
    methods.push({
      kind: "registry_pgp",
      label: uiMessage("auth_method.registry_pgp.label"),
      description:
        pgpFingerprints.length === 1
          ? uiMessage("auth_method.registry_pgp.description_single", {
              fingerprint: pgpFingerprints[0] ?? "",
            })
          : uiMessage("auth_method.registry_pgp.description"),
      ssh_fingerprints: [],
      pgp_fingerprints: pgpFingerprints,
      email_targets: [],
    });
  }

  if (registryEmailEnabled && emailTargets.length > 0) {
    methods.push({
      kind: "registry_email",
      label: uiMessage("auth_method.registry_email.label"),
      description: uiMessage(
        emailTargets.length === 1
          ? "auth_method.registry_email.description_single"
          : "auth_method.registry_email.description",
        {
          emails: emailTargets[0]?.emails.join(", ") ?? "",
        },
      ),
      ssh_fingerprints: [],
      pgp_fingerprints: [],
      email_targets: emailTargets,
    });
  }

  methods.push(...oidcMethods);
  return methods;
}
