# DN42 Autopeer Workflow Survey

Last verified: 2026-04-18

## Purpose

This document summarizes public DN42 autopeer systems and their workflows to inform the `bird-lg-rs` autopeer design. The focus is on:

- How users start peering
- How systems prove ASN ownership
- How node selection works
- What data is collected before provisioning
- How sessions are created, updated, and removed

This survey only uses public pages, public repositories, or public client bundles visible without operator access.

## Workflow Families

Current DN42 autopeer systems broadly fall into five patterns:

1. Web portal with registry-backed authentication and session management
2. SSH self-service CLI
3. Chat bot with per-node agents
4. Signed web form that directly generates config
5. Latency-first node selector with follow-up peering flow

The newer systems tend to split `authentication`, `node selection`, `session creation`, and `session lifecycle management` into separate stages. Older systems often do all of that in one form submission.

## Live-Verified UX Signals

The following public sources were spot-checked live on 2026-04-18 to confirm which workflow ideas are still current:

| Source | Live evidence | UX signal |
| --- | --- | --- |
| DN42 wiki automatic peering index | Recommends using Pingfinder to find the lowest-latency network and lists current fully self-service systems including RoutedBits, FLIPFLAP-DN42, TECH9, Potat0, Nedifinita, and Kioubit | node choice is a first-class step, not a hidden field |
| Kioubit Network | Public landing page exposes a large autopeer-capable node inventory and says the peering dashboard makes the process easy “with a few clicks”; Kioubit Auth is presented as a separate identity service | authentication can be reusable, while node discovery stays public |
| RoutedBits | Login page tells first-time users to authenticate with Kioubit; public nodes page lists location, endpoint, DN42 addresses, and speedtest links | public node directory plus delegated auth keeps the portal focused |
| FLIPFLAP-DN42 | `dn42-sshd-autopeer` slides show an SSH CLI with `peer create`, `peer config`, `peer list`, `peer remove`, and `peer status` | create/update/list/delete are separate lifecycle surfaces, not one overloaded form |
| TECH9 | Public peering page advertises a new self-serve SSH system, says setup is immediate, and lists available locations | a narrow, location-first flow is still viable when the required data is small |
| Potat0 | Public network page says AutoPeer is provided by a Telegram bot, documents per-node requirements, and still lists the exact peering data needed when manual fallback is required | even bot flows end in a clear reviewable list of fields and node choice |

The common denominator is not the transport surface. Some systems use web, some use SSH, and some use Telegram. The stable UX shape is:

1. Expose candidate nodes and help users choose one.
2. Authenticate or delegate authentication.
3. Collect a small required set of tunnel fields.
4. Review the request before provisioning or Git mutation.
5. Show long-lived session state separately from the create form.

## Systems Survey

| System | Public entrypoint | Auth flow | Node selection | Session config flow | Provisioning model | Lifecycle surface |
| --- | --- | --- | --- | --- | --- | --- |
| Kioubit auth / RoutedBits | Web | ASN -> registry-derived auth methods -> email / SSH / PGP / logincode | Public node list with speedtests | Not fully public after login | Likely session-backed portal | Public nodes; post-login flow not fully public |
| Nedifinita | Web SPA | Site password, email, SSH, PGP, Kioubit open-auth | Node list in UI | Wizard with interface + BGP features + review | Session states include auto and reviewed flows | Manage page with status-driven lifecycle |
| FLIPFLAP-DN42 | SSH CLI | SSH login as DN42 maintainer using registry SSH keys | User chooses target SSH node before login | Interactive shell prompts for AS, WG key, endpoint, port, optional link-local | Stored in SQLite, config generated later | List, show config, status, remove |
| TECH9 | SSH CLI | SSH to `cli.dn42.tech9.io` | User picks one of several named sites | Public page says immediate WireGuard creation | Immediate setup | Public details are thin |
| moe233 | SSH CLI | SSH to per-node port `4242` | User chooses nearest published node | Public page documents per-node endpoint and deterministic port scheme | Immediate/self-serve | Legacy migrations handled manually |
| Potat0 bot | Telegram bot | `/login` -> ASN -> registry email code | Bot checks eligible nodes and offers choices | Conversational wizard for families, MP-BGP, addresses, endpoint, key, contact | Bot server posts JSON to node agents that write WG/BIRD config | Create, modify, restart, remove, info |
| KusakabeShi `DN42-AutoPeer` | Web form | ASN -> signed challenge via SSH key or PGP | Single-node or operator-chosen service | Fill tunnel data in one form | Generates WG/BIRD files, syncs Git state, runs `birdc configure` | Show, update, delete by `PeerID` |
| MARAUN | Web | Not public from landing page | Starts with latency check against candidate nodes | Follow-up flow not fully public | Not public | Not public |

## Detailed Notes

### Kioubit auth and RoutedBits

Kioubit exposes a reusable DN42 authentication service:

1. Enter ASN
2. Query registry-backed auth methods
3. Pick an auth mechanism
4. Complete one of:
   - email verification
   - SSH validation
   - PGP clear-sign
   - reusable logincode
5. Return to the requesting service

RoutedBits appears to use this model for first-time authentication. Its public `nodes` page also exposes:

- a large node inventory
- region grouping
- endpoint hostnames
- DN42 tunnel addresses
- transport type hints
- speedtest links

Takeaway: `authentication` can be a reusable service, while `node selection` can stay public and discoverable before a user starts a session.

### Nedifinita

Nedifinita is the clearest example of a modern autopeer web portal. Public frontend routes and strings show this shape:

1. Sign in
2. Select node
3. Choose transport and BGP features
4. Enter tunnel/interface details
5. Review
6. Wait for setup or review
7. Manage sessions later

The public client bundle shows support for:

- login by site password
- email verification
- SSH verification
- PGP verification
- Kioubit open-auth

The public strings also show explicit session states:

- deleted
- disabled
- enabled
- pending approval
- queued for setup
- queued for delete
- problem
- teardown

Takeaway: newer systems model peering as a long-lived session object with explicit state, not as a one-shot config download.

Note: the Nedifinita workflow above is inferred from its public SPA bundle and route structure, not from a public backend repository.

### FLIPFLAP-DN42

FLIPFLAP uses a custom SSH shell on port `4242`:

1. User chooses a node and connects with SSH
2. Username is the lowercase maintainer name without `-MNT`
3. Server authenticates against `auth` keys from the DN42 registry
4. User runs shell commands such as:
   - `peer_create`
   - `peer_config`
   - `peer_list`
   - `peer_remove`
   - `peer_status`
5. `peer_create` asks for:
   - target AS
   - WireGuard public key
   - endpoint address
   - endpoint port
   - optional link-local IPv6
6. Request is written to SQLite
7. Separate config generation consumes the DB and writes WG/BIRD config

Public policy is opinionated:

- WireGuard only
- link-local IPv6 on each peering link
- MP-BGP over IPv6
- extended next-hop over IPv6

Takeaway: SSH is a viable trust boundary when registry SSH keys already exist, and a CLI can keep the workflow narrow and operational.

### TECH9 and moe233

These systems expose self-serve SSH entrypoints with less public implementation detail.

Shared traits:

- user selects a specific node first
- session creation is described as immediate
- WireGuard is the primary transport
- public docs emphasize location choice and endpoint details

Distinct trait for `moe233`:

- the public page documents a deterministic port scheme using the last five digits of the peer ASN

Takeaway: some operators prefer a very thin surface that assumes peers already know the data they need to enter.

### Potat0 Telegram bot

Potat0 splits the workflow between a chat bot and node-local agents.

Login flow:

1. `/login`
2. Enter ASN
3. Bot looks up registry email addresses
4. User picks a registered email
5. Bot sends a verification code
6. User enters the code

Peering flow:

1. `/peer`
2. Bot checks which nodes are open, under capacity, and allowed for the requester
3. User selects region/node
4. User selects route families and MP-BGP behavior
5. User enters DN42 addressing, endpoint, port, WG public key, and contact
6. Bot shows a final review and requires explicit confirmation
7. Bot POSTs JSON to the selected node agent

Provisioning flow inside the agent:

- write `/etc/wireguard/dn42-<asn>.conf`
- write `/etc/bird/dn42_peers/<asn>.conf`
- enable and restart `wg-quick`
- run `birdc c`
- optionally update `vnstat`

Takeaway: a central control plane plus per-node provisioning agents works well when nodes are operationally independent.

### KusakabeShi `DN42-AutoPeer`

This older web-form implementation is still useful because it makes the full request path public.

Flow:

1. Enter ASN
2. Click `Get Signature`
3. Service discovers registry auth methods
4. Service issues a short-lived JWT challenge
5. User signs it with:
   - `ssh-keygen -Y sign`, or
   - `gpg --clearsign`
6. User fills tunnel and BGP fields
7. Click `Register`
8. Backend verifies ASN ownership
9. Backend validates parameters and allocates `PeerID`
10. Backend generates:
    - WireGuard config
    - BIRD config
    - helper shell script
    - peer YAML state
11. Backend syncs Git-backed state and runs `birdc configure`

The same surface also supports:

- `Show`
- `Update`
- `Delete`

Takeaway: signed challenge flow is a good fit for DN42 registry semantics, but the one-page form becomes crowded once the system needs richer lifecycle behavior.

### MARAUN

MARAUN’s public page starts with:

- user public IP or DNS
- user ASN
- a latency check
- instruction to wait a few seconds and repeat the action

Takeaway: some systems optimize `where should you peer?` before they ask `how should the tunnel be configured?`

## Common Workflow Stages

Across the surveyed systems, the stable workflow stages are:

1. Discover candidate nodes
2. Prove control of the DN42 ASN or maintainer
3. Collect tunnel and routing parameters
4. Validate policy and capacity constraints
5. Materialize a session object
6. Generate node-local config
7. Expose status and lifecycle actions

### Identity Proof Patterns

The most common DN42 identity proof methods are:

- registry email code
- registry SSH key validation
- PGP clear-sign against registry keys

Less common but increasingly useful:

- reusable local login code
- federated auth delegation to another DN42 provider
- local site password after initial verification

### Node Selection Patterns

Three node-selection approaches show up repeatedly:

- public node directory first, then auth
- auth first, then choose from eligible nodes
- latency probe first, then continue with a narrowed candidate set

### Session Data Commonly Collected

Most real systems eventually need more than:

- ASN
- IPv4
- IPv6
- endpoint
- comment

Common fields include:

- router or node identifier
- transport type
- WireGuard public key
- optional WireGuard preshared key
- endpoint host or IP
- endpoint port
- IPv6 link-local
- whether IPv4 is enabled
- whether IPv6 ULA is enabled
- whether IPv6 link-local is preferred
- MP-BGP support
- extended next-hop support
- MTU
- routing policy
- operator contact

### Session Lifecycle Patterns

Newer systems generally include explicit lifecycle actions:

- list sessions
- inspect rendered/local config
- update editable parameters
- disable or enable
- remove
- restart or requeue provisioning
- inspect status or metrics

## Implications for `bird-lg-rs`

The `autopeer-worker` backend and the separate `autopeer` frontend now already have the right high-level contract:

- explicit auth/session wire models in [common/src/auto_peer.rs](/Users/iris/Projects/Rust/bird-lg-rs/common/src/auto_peer.rs:1)
- node discovery and session listing from the worker
- explicit operation states for PR creation, checks, merge, and apply
- a dedicated `autopeer` app boundary instead of LG-shell coupling

The main problem was not missing backend shape. It was that the peer-config UX still collapsed too many jobs into one screen.

### Revised Flow Direction

The revised peer-config flow now follows the live survey more closely:

1. `Choose Node`
   - node selection is now its own stage with region/country/IP-support context before any tunnel fields appear
2. `Configure Session`
   - the main form now emphasizes the required fields first: endpoint, WireGuard public key, link-local IPv6, and families
   - advanced values such as comment, port override, keepalive, MTU, and `own6` are moved into a separate advanced section
3. `Review And Submit`
   - the UI now shows a final summary of the peer change before opening the GitHub PR
   - the review copy explains which repo file is touched and which CI/apply stages follow
4. `Lifecycle In Sidebar`
   - active auth state, optional host-ASN impersonation, current operation progress, and existing sessions are moved out of the main form and into a separate status rail

### Why This Direction Fits The Survey

- It matches the public-node-directory-first pattern seen in Kioubit and RoutedBits.
- It matches the separate lifecycle surface seen in FLIPFLAP’s `peer list` / `peer status` commands.
- It keeps support/operator impersonation off the main peering path.
- It keeps the PR/apply lifecycle visible without forcing the user to parse it while entering tunnel data.

This is an explicit move away from the old one-screen shell form where authentication context, session inventory, impersonation, operation status, and raw config fields all competed for the same visual priority.

## Sources

- DN42 automatic peering directory: <https://dn42.cc/wiki/services/automatic-peering/>
- Kioubit auth flow: <https://dn42.g-load.eu/auth/>
- Kioubit public network page: <https://dn42.g-load.eu/>
- Kioubit network/about page: <https://dn42.g-load.eu/about/network/>
- RoutedBits login: <https://dn42.routedbits.io/peering>
- RoutedBits nodes: <https://dn42.routedbits.io/nodes>
- RoutedBits DN42 auth strategy: <https://github.com/routedbits/omniauth-dn42>
- FLIPFLAP public DN42 page: <https://hcartiaux.github.io/dn42/>
- FLIPFLAP SSH autopeer repo: <https://github.com/hcartiaux/dn42-sshd-autopeer>
- FLIPFLAP FOSDEM 2026 slides: <https://fosdem.org/2026/events/attachments/RPJHYK-automating_bgp_peerings_in_the_dn42_environment/slides/267193/fosdem_20_atjbxl8.pdf>
- TECH9 public DN42 peering page: <https://www.chrismoos.com/dn42-peering/>
- moe233 DN42 peering page: <https://blog.moe233.net/dn42/>
- Potat0 DN42 network page: <https://dn42.potat0.cc/>
- Potat0 Telegram bot repo: <https://github.com/Potat0000/dn42-bot>
- KusakabeShi DN42-AutoPeer repo: <https://github.com/KusakabeShi/DN42-AutoPeer>
- Nedifinita portal: <https://peer-dn42.nedifinita.com/>
- Nedifinita public frontend bundle: <https://peer-dn42.nedifinita.com/assets/index-CrS8tKmh.js>
- MARAUN peering page: <https://peering.maraun.de/>
