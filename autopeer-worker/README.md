# `autopeer-worker`

Cloudflare Worker backend for the `bird-lg-rs` autopeer UI.

It handles:

- DN42 registry-backed authentication
- host-ASN-gated impersonation
- repo-backed session discovery from `el-ev/network`
- canonical `host_vars/*/dn42_peers.yaml` mutation
- GitHub PR creation and bot-driven merge after checks pass
- operation tracking in D1 until CI/apply completes

It does not provision peers directly. Deployment still flows through the network repo's existing GitHub Actions:

- `peer-session-check.yml`
- `peer-session-apply.yml`

## Architecture

The Worker uses three state sources:

1. DN42 registry
   - reads `aut-num -> mnt-by -> mntner -> auth`
   - supports registry SSH signatures, registry PGP signatures, and configured OIDC providers
2. GitHub network repo
   - reads `inventory.yaml`
   - optionally reads `group_vars/all/autopeer.yaml`
   - reads and rewrites `host_vars/<node>/dn42_peers.yaml`
3. D1
   - stores short-lived auth challenges
   - stores short-lived authenticated sessions
   - stores async operation state and PR/workflow links

Canonical session truth stays in GitHub, not D1.

## Runtime Config

Wrangler config lives in [wrangler.toml](./wrangler.toml).

Non-secret vars:

- `GITHUB_OWNER`
- `GITHUB_REPO`
- `GITHUB_BASE_BRANCH`
- `DN42_REGISTRY_OWNER`
- `DN42_REGISTRY_REPO`
- `DN42_REGISTRY_BRANCH`
- `DN42_REGISTRY_BASE_URL`
- `OIDC_PROVIDERS`
- `HOST_ASNS`
  - comma-separated ASN list allowed to impersonate other ASNs after authenticating through the normal flow
- `AUTOPEER_API_URL`
- `AUTOPEER_SITE_URL`
- `LOOKING_GLASS_URL`

Secrets you must provide:

- `GITHUB_TOKEN`
  - GitHub token used for repo reads, branch creation, PR creation, and pull request merge API calls
  - for this workflow, use a fine-grained token scoped to the target repository with at least `Metadata: read`, `Contents: read/write`, `Pull requests: read/write`, and `Actions: read`
  - GitHub requires a valid `User-Agent` header on every API request; the Worker now sends one
  - this repo is currently `el-ev/network`, which is user-owned, not org-owned
  - fine-grained PATs can only access repositories owned by the token owner or by an organization the token owner is a member of
  - that means a collaborator account's fine-grained PAT will not work for `el-ev/network`; use a token from `el-ev`, move the repo to an organization, or switch the integration to a GitHub App
- `DN42_GIT_TOKEN`
  - token used to read the DN42 registry through the Gitea API
- any OIDC client secret named by `OIDC_PROVIDERS[].client_secret_env`
  - for Kioubit, this is typically a client created with `client_secret_post`
  - configure each one with `wrangler secret put <name>`

## D1 Schema

Initial schema is in [migrations/0001_init.sql](./migrations/0001_init.sql), with OIDC flow state added in [migrations/0002_oidc_auth.sql](./migrations/0002_oidc_auth.sql).

Tables:

- `auth_challenges`
- `auth_sessions`
- `oidc_auth_requests`
- `operations`

## Host ASN Impersonation

Normal users must authenticate through DN42 registry auth or configured OIDC.

Configured host ASN users authenticate through the normal flow first, then impersonate other ASNs with:

- `POST /v1/auth/impersonate`
- `Authorization: Bearer <session_token>`

Body:

```json
{
  "asn": "4242421234",
  "effective_mnt": "EXAMPLE-MNT"
}
```

Behavior:

- the caller must already hold a valid authenticated session
- the caller's authenticated ASN must be listed in `HOST_ASNS`
- the Worker still resolves maintainers from the DN42 registry
- if the ASN has one maintainer, `effective_mnt` may be omitted
- if the ASN has multiple maintainers, `effective_mnt` is required
- the minted session is then used like any other authenticated autopeer session

## API

Health:

- `GET /health`

Authentication:

- `POST /v1/auth/start`
- `POST /v1/auth/verify/registry-ssh`
- `POST /v1/auth/verify/registry-pgp`
- `POST /v1/auth/oidc/:provider/start`
- `GET /oidc/callback/:provider`
- `POST /v1/auth/oidc/complete`
- `POST /v1/auth/impersonate`

Session management:

- `GET /v1/sessions`
- `POST /v1/sessions`
- `PATCH /v1/sessions/:node/:asn`
- `DELETE /v1/sessions/:node/:asn`
- `POST /v1/sessions/:node/:asn/migrate`

Operation polling:

- `GET /v1/operations/:id`

All session and operation endpoints require:

- `Authorization: Bearer <session_token>`

## Repo Mutation Rules

The Worker only manages autopeer-marked entries.

Current behavior:

- create/update/delete only target `host_vars/<node>/dn42_peers.yaml`
- peer entries are normalized into the same key order used by the network repo tooling
- delete uses `removed: true`, not hard deletion
- migrate converts an existing manual peer into an autopeer-managed peer without changing its session fields, and stamps `auth_provider: migration`
- manual entries are treated as read-only conflicts
- pending D1 operations are overlaid on top of repo-backed sessions when listing state

## Local Development

Install dependencies:

```bash
npm install
```

Generate Worker types after changing `wrangler.toml`:

```bash
npm run types
```

Run local dev:

```bash
npm run dev
```

Typecheck:

```bash
npm run check
```

Run tests:

```bash
npm test
```

Deploy:

```bash
npm run deploy
```

## OIDC Providers

`OIDC_PROVIDERS` is a JSON array string. Each entry looks like:

```json
[
  {
    "name": "kioubit",
    "label": "Kioubit",
    "issuer": "https://dn42.g-load.eu",
    "client_id": "5b6a7a8b-9783-4f9d-9484-091f235ef8dd",
    "client_secret_env": "KIOUBIT_OIDC_CLIENT_SECRET",
    "token_endpoint_auth_method": "client_secret_post",
    "audience": "5b6a7a8b-9783-4f9d-9484-091f235ef8dd",
    "scopes": ["openid", "profile", "email", "dn42"],
    "asn_claim": "dn42.asn",
    "mntner_claim": "dn42.mnt",
    "description": "Authenticate with Kioubit."
  },
  {
    "name": "iedon",
    "label": "iEdon",
    "issuer": "https://auth.iedon.net",
    "client_id": "5fd911be16a3859f0f6603f0cd9e5181",
    "client_secret_env": "IEDON_OIDC_CLIENT_SECRET",
    "token_endpoint_auth_method": "client_secret_post",
    "audience": "5fd911be16a3859f0f6603f0cd9e5181",
    "scopes": ["openid", "profile", "email", "dn42"],
    "asn_claim": "dn42.asn",
    "mntner_claim": "dn42.mnt",
    "description": "Authenticate with iEdon."
  }
]
```

The real flow is authorization-code + PKCE:

1. the frontend asks the worker for an authorization URL
2. the browser redirects to the provider
3. the provider returns to `/oidc/callback/:provider`
4. the worker exchanges the code, verifies the ID token, fetches userinfo if needed, and mints the Autopeer session
5. the frontend completes a one-shot handoff with `POST /v1/auth/oidc/complete`

Provider notes:

- `client_id` is required
- `client_secret_env` is required for `client_secret_post` and `client_secret_basic`
- `token_endpoint_auth_method` defaults to `client_secret_post`
- `scopes` defaults to `["openid", "profile", "email"]`
- `jwks_uri`, `authorization_endpoint`, `token_endpoint`, `userinfo_endpoint`, and `discovery_url` are optional overrides; otherwise the worker uses OIDC discovery from `issuer`
- `asn_claim` and `mntner_claim` support dotted paths such as `dn42.asn`

The verified `mntner` still has to match one of the maintainers resolved from `aut-num -> mnt-by`.

Register the redirect URIs:

```text
https://autopeer.owo.li/oidc/callback/kioubit
https://autopeer.owo.li/oidc/callback/iedon
```

## Validation

Current project checks:

```bash
npm run check
npm test
```

The Rust frontend/backend integration is validated from the repo root with:

```bash
cargo test --workspace
```
