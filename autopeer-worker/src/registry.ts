import { createHash } from "node:crypto";

import type { AuthMethod, MaintainerRecord, RegistryEmailTarget } from "./types";
import { fromBase64, joinPath, readSecret } from "./utils";

interface GiteaContentResponse {
  content?: string;
  encoding?: string;
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

async function fetchRegistryContent(env: Env, path: string): Promise<string> {
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
    throw new Error(`Registry path not found: ${path}`);
  }
  if (!response.ok) {
    throw new Error(`Registry request failed for ${path}: HTTP ${response.status}`);
  }

  const body = (await response.json()) as GiteaContentResponse;
  if (body.encoding !== "base64" || typeof body.content !== "string") {
    throw new Error(`Registry API returned unexpected payload for ${path}`);
  }

  return fromBase64(body.content);
}

async function fetchOptionalRegistryContent(env: Env, path: string): Promise<string | null> {
  try {
    return await fetchRegistryContent(env, path);
  } catch (error) {
    if (error instanceof Error && error.message === `Registry path not found: ${path}`) {
      return null;
    }
    throw error;
  }
}

async function loadContactEmailsForHandle(env: Env, handle: string): Promise<string[]> {
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

export async function loadMaintainersForAsn(env: Env, asn: string): Promise<MaintainerRecord[]> {
  const autNumText = await fetchRegistryContent(env, `data/aut-num/AS${asn}`);
  const maintainerNames = [...new Set(parseFieldValues(autNumText, "mnt-by"))];

  if (maintainerNames.length === 0) {
    throw new Error(`AS${asn} does not expose any mnt-by records in the registry`);
  }

  const maintainers = await Promise.all(
    maintainerNames.map(async (name) => {
      const text = await fetchRegistryContent(env, `data/mntner/${name}`);
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
): AuthMethod[] {
  const methods: AuthMethod[] = [];
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
      label: "Registry SSH Signature",
      description: "Sign our challenge with an SSH key from your DN42 maintainer object.",
      ssh_fingerprints: sshFingerprints,
      pgp_fingerprints: [],
      email_targets: [],
    });
  }

  if (pgpFingerprints.length > 0) {
    methods.push({
      kind: "registry_pgp",
      label: "Registry PGP Signature",
      description: `Use one of your registry PGP fingerprints: ${pgpFingerprints.join(", ")}`,
      ssh_fingerprints: [],
      pgp_fingerprints: pgpFingerprints,
      email_targets: [],
    });
  }

  if (emailTargets.length > 0) {
    methods.push({
      kind: "registry_email",
      label: "Registry Email",
      description: emailTargets.length === 1
        ? `Send a sign-in link and one-time code to ${emailTargets[0].emails.join(", ")}.`
        : "Choose a maintainer and send a sign-in link to its registry email contacts.",
      ssh_fingerprints: [],
      pgp_fingerprints: [],
      email_targets: emailTargets,
    });
  }

  methods.push(...oidcMethods);
  return methods;
}
