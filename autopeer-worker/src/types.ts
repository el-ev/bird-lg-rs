export type AuthMethodKind =
  | "registry_ssh"
  | "registry_pgp"
  | "registry_email"
  | "oidc"
  | "host_impersonation";

export interface UiMessage {
  key: string;
  params?: Record<string, string>;
}

export interface RegistryEmailTarget {
  maintainer: string;
  emails: string[];
}

export interface AuthMethod {
  kind: AuthMethodKind;
  label: UiMessage;
  description: UiMessage;
  provider?: string;
  ssh_fingerprints?: string[];
  pgp_fingerprints?: string[];
  email_targets?: RegistryEmailTarget[];
}

export interface AuthStartRequest {
  asn: string;
}

export interface AuthStartResponse {
  asn: string;
  challenge_id: string;
  challenge_text: string;
  challenge_ttl_seconds: number;
  methods: AuthMethod[];
}

export interface RegistrySshVerifyRequest {
  challenge_id: string;
  signature: string;
}

export interface RegistryPgpVerifyRequest {
  challenge_id: string;
  public_key: string;
  signed_message: string;
}

export interface PgpKeyLookupResponse {
  fingerprint: string;
  found: boolean;
  public_key?: string;
  source?: string;
}

export interface RegistryEmailSendRequest {
  challenge_id: string;
  effective_mnt?: string | null;
  locale?: string | null;
}

export interface RegistryEmailSendResponse {
  effective_mnt: string;
  emails: string[];
  expires_at: string;
}

export interface RegistryEmailVerifyRequest {
  challenge_id: string;
  code: string;
}

export interface RegistryEmailCompleteRequest {
  token: string;
}

export interface OidcStartRequest {
  challenge_id?: string;
}

export interface OidcStartResponse {
  authorization_url: string;
}

export interface OidcCompleteRequest {
  state: string;
}

export interface HostImpersonationRequest {
  asn: string;
  effective_mnt?: string | null;
}

export interface AuthSessionResponse {
  session_token: string;
  asn: string;
  effective_mnt: string;
  auth_method: AuthMethod;
  can_impersonate: boolean;
  expires_at: string;
}

export interface PeeringInfo {
  ipv4?: string;
  ipv6?: string;
  link_local_ipv6?: string;
  wg_pubkey?: string;
  endpoint?: string;
  comment?: string;
}

export interface NodeView {
  name: string;
  endpoint_host?: string;
  region?: string;
  country?: string;
  ip_support: string;
  comment?: string;
  peering?: PeeringInfo;
  autopeer?: boolean;
}

export type SessionState = "managed" | "manual" | "locked" | "pending_pr" | "stalled_pr" | "conflict";

export interface SessionMetadata {
  managed: boolean;
  effective_mnt?: string;
  auth_provider?: string;
}

export const PEERING_STRATEGIES = [
  "full_table",
  "transit",
  "peer",
  "downstream",
] as const;

export type PeeringStrategy = (typeof PEERING_STRATEGIES)[number];
export const MP_BGP_TRANSPORTS = ["ipv4", "ipv6"] as const;
export type MpBgpTransport = (typeof MP_BGP_TRANSPORTS)[number];

export interface PeerSessionSpec {
  comment?: string | null;
  endpoint?: string | null;
  wg_public_key: string;
  port?: number | null;
  peer4?: string | null;
  peer6?: string | null;
  own6?: string | null;
  keepalive?: number | null;
  mtu?: number | null;
  ipv4: boolean;
  ipv6: boolean;
  extended_next_hop: boolean;
  mp_bgp: boolean;
  mp_bgp_transport?: MpBgpTransport | null;
  peering_strategy: PeeringStrategy;
  psk?: string | null;
  has_psk?: boolean;
  encrypt_endpoint?: boolean;
}

export interface SessionView {
  node: string;
  asn: string;
  state: SessionState;
  spec?: PeerSessionSpec;
  metadata?: SessionMetadata;
  has_psk?: boolean;
  has_encrypted_endpoint?: boolean;
  pending_operation_id?: string;
  pull_request_url?: string;
  message?: UiMessage;
}

export interface SessionListResponse {
  asn: string;
  nodes: NodeView[];
  sessions: SessionView[];
}

export interface CreateSessionRequest {
  node: string;
  session: PeerSessionSpec;
}

export interface UpdateSessionRequest {
  session: PeerSessionSpec;
}

export type OperationKind = "create" | "update" | "retire" | "delete" | "migrate";
export type OperationState =
  | "pending_pull_request"
  | "pending_checks"
  | "applying"
  | "pending_merge"
  | "completed"
  | "failed"
  | "conflict";

export interface OperationFailureDetails {
  stage: "checks" | "preflight" | "apply" | "merge";
  step?: string | null;
  conclusion?: string | null;
  run_url?: string | null;
  annotation?: string | null;
}

export interface OperationStatus {
  id: string;
  asn: string;
  node: string;
  kind: OperationKind;
  state: OperationState;
  branch: string;
  pr_number?: number | null;
  pull_request_url?: string | null;
  workflow_run_url?: string | null;
  message?: UiMessage | null;
  failure_details?: OperationFailureDetails | null;
  created_at: string;
  updated_at: string;
}

export interface ApiError {
  error: UiMessage;
}

export interface MaintainerRecord {
  name: string;
  auth_lines: string[];
  ssh_public_keys: string[];
  ssh_fingerprints: string[];
  pgp_fingerprints: string[];
  contact_emails: string[];
}

export interface ChallengeRecord {
  id: string;
  asn: string;
  challenge_text: string;
  methods: AuthMethod[];
  maintainers: MaintainerRecord[];
  created_at: string;
  expires_at: string;
}

export interface SessionRecord {
  token: string;
  asn: string;
  effective_mnt: string;
  auth_method: AuthMethod;
  created_at: string;
  expires_at: string;
}

export interface OperationRecord extends OperationStatus {
  pr_node_id?: string | null;
  session_snapshot?: PeerSessionSpec | null;
}

export type OidcTokenEndpointAuthMethod =
  | "client_secret_post"
  | "client_secret_basic"
  | "none";

export type OidcClaimPath = string | string[];

export interface OidcProviderConfig {
  name: string;
  label: string;
  issuer: string;
  client_id: string;
  client_secret_env?: string;
  audience: string;
  discovery_url?: string;
  authorization_endpoint?: string;
  token_endpoint?: string;
  userinfo_endpoint?: string;
  jwks_uri?: string;
  token_endpoint_auth_method?: OidcTokenEndpointAuthMethod;
  scopes?: string[];
  asn_claim: OidcClaimPath;
  mntner_claim: OidcClaimPath;
  description?: string;
  dn42_issuer?: string;
}

export interface OidcProviderDiscovery {
  issuer: string;
  authorization_endpoint: string;
  token_endpoint: string;
  jwks_uri: string;
  userinfo_endpoint?: string;
}

export interface OidcTokenResponse {
  access_token?: string;
  token_type?: string;
  expires_in?: number;
  refresh_token?: string;
  scope?: string;
  id_token?: string;
}

export interface OidcAuthRequestRecord {
  state: string;
  challenge_id: string;
  provider: string;
  nonce: string;
  code_verifier: string;
  redirect_uri: string;
  session_token?: string | null;
  created_at: string;
  expires_at: string;
}

export interface RegistryEmailAuthRequestRecord {
  challenge_id: string;
  effective_mnt: string;
  email_snapshot: string[];
  code: string;
  token: string;
  session_token?: string | null;
  locale?: string | null;
  created_at: string;
  expires_at: string;
}

export interface InventoryHost {
  name: string;
  endpoint_host?: string;
  region?: string;
  country?: string;
  ip_support: string;
  comment?: string;
  peering?: PeeringInfo;
  autopeer?: boolean;
}
