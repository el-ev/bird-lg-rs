import type { UiMessage } from "dn42-auth-worker/types";
export type {
  AuthMethodKind,
  UiMessage,
  RegistryEmailTarget,
  AuthMethod,
  AuthStartResponse,
  PgpKeyLookupResponse,
  RegistryEmailSendResponse,
  OidcStartResponse,
  AuthSessionResponse,
  MaintainerRecord,
  ChallengeRecord,
  SessionRecord,
  OidcTokenEndpointAuthMethod,
  OidcClaimPath,
  OidcProviderConfig,
  OidcProviderDiscovery,
  OidcTokenResponse,
  OidcAuthRequestRecord,
  RegistryEmailAuthRequestRecord,
} from "dn42-auth-worker/types";

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
  stalled_notified_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ApiError {
  error: UiMessage;
}

export interface OperationRecord extends OperationStatus {
  pr_node_id?: string | null;
  session_snapshot?: PeerSessionSpec | null;
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

export type {
  AuthStartRequest,
  HostImpersonationRequest,
  RegistrySshVerifyRequest,
  RegistryPgpVerifyRequest,
  RegistryEmailSendRequest,
  RegistryEmailVerifyRequest,
  RegistryEmailCompleteRequest,
  OidcStartRequest,
  OidcCompleteRequest,
} from "dn42-auth-worker/schemas";
export type {
  CreateSessionRequest,
  UpdateSessionRequest,
} from "./schemas";
