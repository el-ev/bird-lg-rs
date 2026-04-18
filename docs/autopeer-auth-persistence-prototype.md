# Autopeer Auth and Persistence Prototype

Last updated: 2026-04-18

This prototype turns the survey in [dn42-autopeer-workflow-survey.md](./dn42-autopeer-workflow-survey.md) into a backend-shaped model without implementing provisioning.

## Scope

- Keep the current mocked `init -> challenge -> verify -> manage sessions` flow.
- Add explicit authentication session records and persisted peering session ownership.
- Model setup and deletion as lifecycle states only.
- Leave config generation, command execution, and node-local reconciliation out of scope until the executor exists.

## Authentication Model

The new shared auth types in [common/src/auto_peer.rs](/Users/iris/Projects/Rust/bird-lg-rs/common/src/auto_peer.rs:1) split authentication into three records:

- `AutoPeerAuthSession`
  - Tracks `asn`, allowed challenge methods, selected method, current auth state, and timestamps.
  - Represents the short-lived workflow from ASN entry to successful verification.
- `AutoPeerChallenge`
  - Captures the issued challenge plus optional delivery target and expiry.
  - Works for both inline signed challenges and out-of-band email delivery.
- `AutoPeerCredential`
  - Represents the verified capability that can later authorize session CRUD.
  - Keeps the verified ASN and method separate from the mutable auth-session state.

This matches the survey pattern where ASN ownership proof is distinct from long-lived session management.

## Persistence Model

Peering state is now modeled in two layers:

- `PeeringSession`
  - User-facing session payload.
  - Adds `transport`, `status`, `status_message`, and `details`.
  - `details` carries the minimum useful fields from the survey: `node_id`, split endpoint data, WireGuard key material, IPv6 link-local, family toggles, MP-BGP flags, MTU, policy, and operator contact.
- `StoredPeeringSession`
  - Persistence record that wraps `PeeringSession` with:
    - `session_id`
    - `owner.asn`
    - `owner.credential_id`
    - optional `owner.auth_session_id`
    - `created_at`, `updated_at`, `revision`

The top-level snapshot is `AutoPeerStoreSnapshot`:

- `auth_sessions: BTreeMap<String, AutoPeerAuthSession>`
- `peering_sessions: BTreeMap<String, StoredPeeringSession>`
- `version`

That snapshot can be stored as JSON now and mapped to SQLite later with near-direct tables:

- `auth_sessions`
- `peering_sessions`

## Lifecycle Without Executor

The model intentionally stops at durable state transitions:

- `draft`
- `pending_approval`
- `queued_for_setup`
- `active`
- `disabled`
- `queued_for_delete`
- `problem`
- `deleted`

Until the backend executor exists, `queued_for_setup` and `queued_for_delete` are the handoff points. The executor should consume those records and only mutate `status`, `status_message`, and revisioned timestamps after attempting provisioning.

## Why This Shape

- It preserves the current UI surface while making ownership and lifecycle explicit.
- It supports both JSON snapshot storage and a future SQLite implementation.
- It keeps executor concerns outside `common`, which matches the crate boundary guidance in `AGENTS.md`.
