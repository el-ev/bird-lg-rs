import { describe, expect, it } from "vitest";

import { methodsFromMaintainers, sshPublicKeyFingerprint } from "../src/registry";
import type { MaintainerRecord } from "../src/types";

const registrySshKey =
  "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGstvB481r7QTtmIrQU5OxAO5ky32DPV5/yHDPdEGjMH iris@Iriss-Laptop.local";
const registrySshFingerprint = "SHA256:3UlLs+FpaF8J7Gbqqs1zlQgO8Frkgqs+2mVZHhoIrTs";

describe("registry auth metadata", () => {
  it("derives OpenSSH-style SHA256 fingerprints from registry SSH keys", () => {
    expect(sshPublicKeyFingerprint(registrySshKey)).toBe(registrySshFingerprint);
  });

  it("exposes unique SSH fingerprints through registry SSH auth methods", () => {
    const maintainers: MaintainerRecord[] = [
      {
        name: "EXAMPLE-MNT",
        auth_lines: [registrySshKey],
        ssh_public_keys: [registrySshKey],
        ssh_fingerprints: [registrySshFingerprint],
        pgp_fingerprints: [],
        contact_emails: [],
      },
      {
        name: "EXAMPLE2-MNT",
        auth_lines: [registrySshKey],
        ssh_public_keys: [registrySshKey],
        ssh_fingerprints: [registrySshFingerprint],
        pgp_fingerprints: [],
        contact_emails: [],
      },
    ];

    expect(methodsFromMaintainers(maintainers, [])).toContainEqual({
      kind: "registry_ssh",
      label: "Registry SSH Signature",
      description: "Sign our challenge with an SSH key from your DN42 maintainer object.",
      ssh_fingerprints: [registrySshFingerprint],
      pgp_fingerprints: [],
      email_targets: [],
    });
  });

  it("exposes registry email auth targets grouped by maintainer", () => {
    const maintainers: MaintainerRecord[] = [
      {
        name: "EXAMPLE-MNT",
        auth_lines: [],
        ssh_public_keys: [],
        ssh_fingerprints: [],
        pgp_fingerprints: [],
        contact_emails: ["admin@example.net", "noc@example.net"],
      },
      {
        name: "SECOND-MNT",
        auth_lines: [],
        ssh_public_keys: [],
        ssh_fingerprints: [],
        pgp_fingerprints: [],
        contact_emails: ["ops@example.net"],
      },
    ];

    expect(methodsFromMaintainers(maintainers, [])).toContainEqual({
      kind: "registry_email",
      label: "Registry Email",
      description: "Choose a maintainer and send a sign-in link to its registry email contacts.",
      ssh_fingerprints: [],
      pgp_fingerprints: [],
      email_targets: [
        {
          maintainer: "EXAMPLE-MNT",
          emails: ["admin@example.net", "noc@example.net"],
        },
        {
          maintainer: "SECOND-MNT",
          emails: ["ops@example.net"],
        },
      ],
    });
  });

  it("hides registry email auth when the mailer is unavailable", () => {
    const maintainers: MaintainerRecord[] = [
      {
        name: "EXAMPLE-MNT",
        auth_lines: [],
        ssh_public_keys: [],
        ssh_fingerprints: [],
        pgp_fingerprints: [],
        contact_emails: ["admin@example.net"],
      },
    ];

    expect(
      methodsFromMaintainers(maintainers, [], { registryEmailEnabled: false }),
    ).not.toContainEqual(
      expect.objectContaining({
        kind: "registry_email",
      }),
    );
  });
});
