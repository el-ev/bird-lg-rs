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

export interface AuthStartResponse {
  asn: string;
  challenge_id: string;
  challenge_text: string;
  challenge_ttl_seconds: number;
  methods: AuthMethod[];
}

export interface PgpKeyLookupResponse {
  fingerprint: string;
  found: boolean;
  public_key?: string;
  source?: string;
}

export interface RegistryEmailSendResponse {
  effective_mnt: string;
  emails: string[];
  expires_at: string;
}

export interface OidcStartResponse {
  authorization_url: string;
}

export interface AuthSessionResponse {
  session_token: string;
  asn: string;
  effective_mnt: string;
  auth_method: AuthMethod;
  can_impersonate: boolean;
  expires_at: string;
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
  site_return_url?: string | null;
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
  site_return_url?: string | null;
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
} from "./schemas";
