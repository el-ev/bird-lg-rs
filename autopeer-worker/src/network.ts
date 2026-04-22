import { isMap, isScalar, isSeq, parseDocument } from "yaml";

import {
  MP_BGP_TRANSPORTS,
  PEERING_STRATEGIES,
  type AuthMethod,
  type InventoryHost,
  type MpBgpTransport,
  type NodeView,
  type OperationKind,
  type OperationRecord,
  type PeeringStrategy,
  type PeerSessionSpec,
  type SessionView,
} from "./types";
import { defaultPeerPort, isTruthyRecord, normalizeAsn } from "./utils";
import { isVaultEncrypted, vaultDecrypt, vaultEncrypt } from "./vault";

const PEER_KEY_ORDER = ["comment", "wg", "bgp", "removed"] as const;
const WG_KEY_ORDER = [
  "port",
  "endpoint",
  "wg_pubkey",
  "psk",
  "peer4",
  "peer6",
  "own6",
  "keepalive",
  "mtu",
] as const;
const DEFAULT_PEERING_STRATEGY: PeeringStrategy = "full_table";
const BGP_KEY_ORDER = [
  "asn",
  "ipv4",
  "ipv6",
  "extended_next_hop",
  "mp_bgp",
  "mp_bgp_transport",
  "peering_strategy",
] as const;
const BASE64_LIKE = /^[A-Za-z0-9+/]+={0,2}$/;

export type TaggedScalar = { tag: string; value: string };
type YamlValue =
  | string
  | number
  | boolean
  | null
  | TaggedScalar
  | YamlValue[]
  | YamlObject;
type YamlObject = { [key: string]: YamlValue | undefined };

interface PeerEntry {
  [key: string]: YamlValue | undefined;
  comment?: string | null;
  wg?: YamlObject;
  bgp?: YamlObject;
  removed?: boolean;
  autopeer?: YamlObject;
}

function isTaggedScalar(value: unknown): value is TaggedScalar {
  return isTruthyRecord(value) && typeof value.tag === "string" && typeof value.value === "string";
}

function scalarToValue(value: unknown, tag?: string): YamlValue {
  if (tag && tag.startsWith("!") && typeof value === "string") {
    return { tag, value };
  }
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean" ||
    value === null
  ) {
    return value;
  }
  return value === undefined ? null : String(value);
}

function nodeToValue(node: unknown): YamlValue {
  if (isMap(node)) {
    const result: YamlObject = {};
    for (const item of node.items) {
      const key = String(nodeToValue(item.key));
      result[key] = nodeToValue(item.value);
    }
    return result;
  }
  if (isSeq(node)) {
    return node.items.map(nodeToValue);
  }
  if (isScalar(node)) {
    return scalarToValue(node.value, node.tag);
  }
  return null;
}

function parseYamlObject(text: string): YamlObject {
  const doc = parseDocument(text);
  const root = nodeToValue(doc.contents);
  if (!isTruthyRecord(root)) {
    throw new Error("YAML root must be a mapping");
  }
  return root as YamlObject;
}

function toPlainObject(text: string): Record<string, unknown> {
  const doc = parseDocument(text);
  const value = doc.toJS();
  if (!isTruthyRecord(value)) {
    throw new Error("YAML root must be a mapping");
  }
  return value;
}

function orderedObject(mapping: YamlObject, preferredKeys: readonly string[]): YamlObject {
  const ordered: YamlObject = {};
  for (const key of preferredKeys) {
    if (key in mapping) {
      ordered[key] = mapping[key];
    }
  }
  for (const [key, value] of Object.entries(mapping)) {
    if (!(key in ordered)) {
      ordered[key] = value;
    }
  }
  return ordered;
}

function normalizeGeneric(value: YamlValue | undefined): YamlValue | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (Array.isArray(value)) {
    return value.map((item) => normalizeGeneric(item) ?? null);
  }
  if (isTruthyRecord(value) && !isTaggedScalar(value)) {
    const normalized: YamlObject = {};
    for (const [key, child] of Object.entries(value)) {
      const normalizedChild = normalizeGeneric(child as YamlValue);
      if (normalizedChild !== undefined) {
        normalized[key] = normalizedChild;
      }
    }
    return normalized;
  }
  return value;
}

function coerceInt(value: YamlValue | undefined): number | undefined {
  if (typeof value === "number" && Number.isInteger(value)) {
    return value;
  }
  if (typeof value === "string" && /^-?\d+$/.test(value.trim())) {
    return Number(value.trim());
  }
  return undefined;
}

function normalizeKnownValue(mapping: YamlObject, key: string, defaultValue: YamlValue): YamlValue {
  const normalized = normalizeGeneric(mapping[key]);
  return normalized === undefined ? defaultValue : normalized;
}

function isPeeringStrategy(value: unknown): value is PeeringStrategy {
  return typeof value === "string" &&
    (PEERING_STRATEGIES as readonly string[]).includes(value);
}

function isMpBgpTransport(value: unknown): value is MpBgpTransport {
  return typeof value === "string" &&
    (MP_BGP_TRANSPORTS as readonly string[]).includes(value);
}

function normalizePeeringStrategy(value: YamlValue | undefined, field: string): PeeringStrategy {
  if (value === undefined || value === null) {
    return DEFAULT_PEERING_STRATEGY;
  }
  if (typeof value !== "string") {
    throw new Error(`${field} must be one of ${PEERING_STRATEGIES.join(", ")}`);
  }
  const normalized = value.trim();
  if (!isPeeringStrategy(normalized)) {
    throw new Error(`${field} must be one of ${PEERING_STRATEGIES.join(", ")}`);
  }
  return normalized;
}

function normalizeMpBgpTransport(
  value: YamlValue | undefined,
  field: string,
): MpBgpTransport | undefined {
  if (value === undefined || value === null) {
    return undefined;
  }
  if (typeof value !== "string") {
    throw new Error(`${field} must be one of ${MP_BGP_TRANSPORTS.join(", ")}`);
  }
  const normalized = value.trim();
  if (!isMpBgpTransport(normalized)) {
    throw new Error(`${field} must be one of ${MP_BGP_TRANSPORTS.join(", ")}`);
  }
  return normalized;
}

function normalizePeerEntry(peer: PeerEntry): PeerEntry {
  if (!isTruthyRecord(peer)) {
    throw new Error("peer entry must be a mapping");
  }

  const bgp = isTruthyRecord(peer.bgp) ? (peer.bgp as YamlObject) : null;
  if (!bgp) {
    throw new Error("peer entry is missing bgp mapping");
  }

  const asn = coerceInt(bgp.asn);
  if (asn === undefined) {
    throw new Error("peer entry is missing valid bgp.asn");
  }
  const peerLabel = `peer AS${asn}`;

  const removed = Boolean(peer.removed);
  const wg = isTruthyRecord(peer.wg) ? (peer.wg as YamlObject) : {};
  if (!removed && !isTruthyRecord(peer.wg)) {
    throw new Error(`active peer AS${asn} is missing wg mapping`);
  }

  const normalizedPeer: PeerEntry = {};
  const comment = normalizeGeneric(peer.comment as YamlValue | undefined);
  if (comment !== undefined) {
    normalizedPeer.comment = comment as string | null;
  }

  const rawPort = normalizeKnownValue(wg, "port", defaultPeerPort(String(asn)));
  const normalizedPort = coerceInt(rawPort);
  const normalizedWg: YamlObject = {
    port: normalizedPort ?? rawPort,
    endpoint: normalizeKnownValue(wg, "endpoint", null),
    wg_pubkey: normalizeKnownValue(wg, "wg_pubkey", null),
    psk: normalizeKnownValue(wg, "psk", null),
    peer4: normalizeKnownValue(wg, "peer4", null),
    peer6: normalizeKnownValue(wg, "peer6", null),
    own6: normalizeKnownValue(wg, "own6", null),
    keepalive: normalizeKnownValue(wg, "keepalive", null),
    mtu: normalizeKnownValue(wg, "mtu", null),
  };
  for (const [key, value] of Object.entries(wg)) {
    if (!WG_KEY_ORDER.includes(key as (typeof WG_KEY_ORDER)[number])) {
      const normalizedValue = normalizeGeneric(value as YamlValue);
      if (normalizedValue !== undefined) {
        normalizedWg[key] = normalizedValue;
      }
    }
  }
  if (Object.keys(normalizedWg).length > 0) {
    normalizedPeer.wg = orderedObject(normalizedWg, WG_KEY_ORDER);
  }

  const normalizedBgp: YamlObject = {
    asn,
    ipv4: normalizeKnownValue(bgp, "ipv4", true),
    ipv6: normalizeKnownValue(bgp, "ipv6", true),
    extended_next_hop: normalizeKnownValue(bgp, "extended_next_hop", true),
    mp_bgp: normalizeKnownValue(bgp, "mp_bgp", true),
  };
  const mpBgpTransport = normalizeMpBgpTransport(
    bgp.mp_bgp_transport as YamlValue | undefined,
    `${peerLabel} bgp.mp_bgp_transport`,
  );
  if (mpBgpTransport !== undefined) {
    normalizedBgp.mp_bgp_transport = mpBgpTransport;
  }
  const peeringStrategy = normalizePeeringStrategy(
    bgp.peering_strategy as YamlValue | undefined,
    `${peerLabel} bgp.peering_strategy`,
  );
  if (peeringStrategy !== DEFAULT_PEERING_STRATEGY) {
    normalizedBgp.peering_strategy = peeringStrategy;
  }
  for (const [key, value] of Object.entries(bgp)) {
    if (!BGP_KEY_ORDER.includes(key as (typeof BGP_KEY_ORDER)[number])) {
      const normalizedValue = normalizeGeneric(value as YamlValue);
      if (normalizedValue !== undefined) {
        normalizedBgp[key] = normalizedValue;
      }
    }
  }
  normalizedPeer.bgp = orderedObject(normalizedBgp, BGP_KEY_ORDER);

  if (removed) {
    normalizedPeer.removed = true;
  }

  for (const [key, value] of Object.entries(peer)) {
    if (!PEER_KEY_ORDER.includes(key as (typeof PEER_KEY_ORDER)[number])) {
      const normalizedValue = normalizeGeneric(value as YamlValue);
      if (normalizedValue !== undefined) {
        normalizedPeer[key] = normalizedValue;
      }
    }
  }

  return orderedObject(normalizedPeer, PEER_KEY_ORDER) as PeerEntry;
}

function normalizePeerFileData(data: YamlObject): { peers: PeerEntry[] } {
  const peers = Array.isArray(data.peers) ? data.peers : null;
  if (!peers) {
    throw new Error("peer file must contain a top-level peers list");
  }

  return {
    peers: peers.map((peer) => normalizePeerEntry(peer as PeerEntry)),
  };
}

function quoteString(value: string): string {
  return `'${value.replace(/'/g, "''")}'`;
}

function dumpTaggedScalar(value: TaggedScalar, indent: number): string {
  const normalizedValue = value.value.replace(/\r\n/g, "\n").replace(/\n$/, "");
  const lines = normalizedValue === "" ? [""] : normalizedValue.split("\n");
  const prefix = `${value.tag} |`;
  const body = lines.map((line) => `${" ".repeat(indent + 2)}${line}`).join("\n");
  return `${prefix}\n${body}`;
}

function dumpScalar(value: string | number | boolean | null | TaggedScalar, indent: number): string {
  if (value === null) {
    return "null";
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (isTaggedScalar(value)) {
    if (value.tag === "!vault" || value.value.includes("\n")) {
      return dumpTaggedScalar(value, indent);
    }
    return `${value.tag} ${quoteString(value.value)}`;
  }
  return quoteString(value);
}

function dumpValue(value: YamlValue, indent = 0): string[] {
  if (Array.isArray(value)) {
    const lines: string[] = [];
    for (const item of value) {
      if (isTruthyRecord(item) && !isTaggedScalar(item)) {
        const [first, ...rest] = dumpObject(item as YamlObject, indent + 2);
        const nestedIndent = " ".repeat(indent + 2);
        const firstInline = first.startsWith(nestedIndent) ? first.slice(indent + 2) : first.trimStart();
        lines.push(`${" ".repeat(indent)}- ${firstInline}`);
        for (const line of rest) {
          lines.push(line);
        }
      } else {
        lines.push(`${" ".repeat(indent)}- ${dumpScalar(item as any, indent)}`);
      }
    }
    return lines;
  }

  if (isTruthyRecord(value) && !isTaggedScalar(value)) {
    return dumpObject(value as YamlObject, indent);
  }

  return [`${" ".repeat(indent)}${dumpScalar(value as any, indent)}`];
}

function dumpObject(value: YamlObject, indent = 0): string[] {
  const lines: string[] = [];
  for (const [key, child] of Object.entries(value)) {
    if (child === undefined) {
      continue;
    }
    if (isTruthyRecord(child) && !isTaggedScalar(child) && !Array.isArray(child)) {
      lines.push(`${" ".repeat(indent)}${key}:`);
      lines.push(...dumpObject(child as YamlObject, indent + 2));
      continue;
    }
    if (Array.isArray(child)) {
      lines.push(`${" ".repeat(indent)}${key}:`);
      lines.push(...dumpValue(child, indent + 2));
      continue;
    }
    const rendered = dumpScalar(child as any, indent);
    if (rendered.includes("\n")) {
      const [first, ...rest] = rendered.split("\n");
      lines.push(`${" ".repeat(indent)}${key}: ${first}`);
      for (const line of rest) {
        lines.push(line);
      }
    } else {
      lines.push(`${" ".repeat(indent)}${key}: ${rendered}`);
    }
  }
  return lines;
}

export function dumpPeerYaml(data: { peers: PeerEntry[] }): string {
  return `${dumpObject(data as unknown as YamlObject).join("\n")}\n`;
}

function peerAsn(peer: PeerEntry): string {
  const bgp = isTruthyRecord(peer.bgp) ? (peer.bgp as YamlObject) : {};
  const asn = coerceInt(bgp.asn);
  if (asn === undefined) {
    throw new Error("peer entry is missing valid bgp.asn");
  }
  return String(asn);
}

function isManagedPeer(peer: PeerEntry): boolean {
  return (
    isTruthyRecord(peer.autopeer) &&
    (peer.autopeer as Record<string, unknown>).managed === true
  );
}

function isRemovedPeer(peer: PeerEntry): boolean {
  return Boolean(peer.removed);
}

async function toSpec(peer: PeerEntry, vaultPassword?: string | null): Promise<PeerSessionSpec> {
  const wg = (peer.wg ?? {}) as YamlObject;
  const bgp = (peer.bgp ?? {}) as YamlObject;
  let endpoint = "";
  if (isVaultEncrypted(wg.endpoint)) {
    if (vaultPassword) {
      try {
        endpoint = await vaultDecrypt(wg.endpoint, vaultPassword);
      } catch { /* leave empty if decryption fails */ }
    }
  } else {
    endpoint = String(wg.endpoint ?? "");
  }
  return {
    comment: typeof peer.comment === "string" ? peer.comment : null,
    endpoint,
    wg_public_key: String(wg.wg_pubkey ?? ""),
    port: coerceInt(wg.port) ?? null,
    peer4: typeof wg.peer4 === "string" ? wg.peer4 : null,
    peer6: typeof wg.peer6 === "string" ? wg.peer6 : null,
    own6: typeof wg.own6 === "string" ? wg.own6 : null,
    keepalive: coerceInt(wg.keepalive) ?? null,
    mtu: coerceInt(wg.mtu) ?? null,
    ipv4: bgp.ipv4 !== false,
    ipv6: bgp.ipv6 !== false,
    extended_next_hop: bgp.extended_next_hop !== false,
    mp_bgp: bgp.mp_bgp !== false,
    mp_bgp_transport: isMpBgpTransport(bgp.mp_bgp_transport) ? bgp.mp_bgp_transport : null,
    peering_strategy: normalizePeeringStrategy(
      bgp.peering_strategy as YamlValue | undefined,
      "bgp.peering_strategy",
    ),
  };
}

function peerHasPsk(peer: PeerEntry): boolean {
  const wg = (peer.wg ?? {}) as YamlObject;
  return wg.psk !== null && wg.psk !== undefined;
}

function peerHasEncryptedEndpoint(peer: PeerEntry): boolean {
  const wg = (peer.wg ?? {}) as YamlObject;
  return isVaultEncrypted(wg.endpoint);
}

function specToPeerEntry(
  asn: string,
  effectiveMnt: string,
  authMethod: AuthMethod,
  spec: PeerSessionSpec,
  existing?: PeerEntry,
): PeerEntry {
  const existingWg = isTruthyRecord(existing?.wg) ? (existing?.wg as YamlObject) : {};
  const existingBgp = isTruthyRecord(existing?.bgp) ? (existing?.bgp as YamlObject) : {};
  const existingAutopeer = isTruthyRecord(existing?.autopeer)
    ? (existing?.autopeer as YamlObject)
    : {};

  const authProvider =
    authMethod.provider === "migration"
      ? "migration"
      : authMethod.kind === "oidc"
        ? authMethod.provider ?? "oidc"
        : authMethod.kind;

  const pskValue: YamlValue =
    spec.psk !== undefined ? (spec.psk ?? null) : (existingWg.psk ?? null);

  return normalizePeerEntry({
    ...(existing ?? {}),
    comment: spec.comment ?? undefined,
    wg: {
      ...existingWg,
      port: spec.port ?? defaultPeerPort(asn),
      endpoint: spec.endpoint,
      wg_pubkey: spec.wg_public_key,
      psk: pskValue,
      peer4: spec.peer4 ?? null,
      peer6: spec.peer6 ?? null,
      own6: spec.own6 ?? null,
      keepalive: spec.keepalive ?? null,
      mtu: spec.mtu ?? null,
    },
    bgp: {
      ...existingBgp,
      asn: Number(asn),
      ipv4: spec.ipv4,
      ipv6: spec.ipv6,
      extended_next_hop: spec.extended_next_hop,
      mp_bgp: spec.mp_bgp,
      mp_bgp_transport: spec.mp_bgp_transport ?? undefined,
      peering_strategy:
        spec.peering_strategy === DEFAULT_PEERING_STRATEGY ? undefined : spec.peering_strategy,
    },
    removed: false,
    autopeer: {
      ...existingAutopeer,
      managed: true,
      effective_mnt: effectiveMnt,
      auth_provider: authProvider,
    },
  });
}

function migrationAuthMethod(authMethod: AuthMethod): AuthMethod {
  return {
    ...authMethod,
    provider: "migration",
  };
}

function ensureEndpointAllowed(node: InventoryHost, endpoint: string): void {
  const { kind } = parseEndpoint(endpoint);
  if (kind === "ipv4" && node.ip_support === "ipv6") {
    throw new Error(`${node.name} is IPv6-only; use a hostname or IPv6 endpoint`);
  }
  if (kind === "ipv6" && node.ip_support === "ipv4") {
    throw new Error(`${node.name} is IPv4-only; use a hostname or IPv4 endpoint`);
  }
}

function parseEndpoint(endpoint: string): { host: string; port: number; kind: "hostname" | "ipv4" | "ipv6" } {
  const trimmed = endpoint.trim();
  if (!trimmed) {
    throw new Error("endpoint is required");
  }
  if (/\s/.test(trimmed)) {
    throw new Error("endpoint cannot contain spaces");
  }

  if (trimmed.startsWith("[")) {
    const match = trimmed.match(/^\[([^\]]+)\]:(\d+)$/);
    if (!match) {
      throw new Error("endpoint must use host:port or [ipv6]:port");
    }
    if (!isValidIpv6Address(match[1])) {
      throw new Error("endpoint IPv6 address must be valid");
    }
    return {
      host: match[1],
      port: parsePortComponent(match[2], "endpoint port"),
      kind: "ipv6",
    };
  }

  if (trimmed.split(":").length !== 2) {
    throw new Error("endpoint must use host:port or [ipv6]:port");
  }

  const [host, portText] = trimmed.split(":");
  if (!host) {
    throw new Error("endpoint host is required");
  }
  const port = parsePortComponent(portText, "endpoint port");
  if (isValidIpv4Address(host)) {
    return { host, port, kind: "ipv4" };
  }
  if (!isValidEndpointHostname(host)) {
    throw new Error("endpoint host must be an IPv4 address or fully qualified hostname");
  }
  return { host, port, kind: "hostname" };
}

function parsePortComponent(value: string, label: string): number {
  const port = Number(value);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error(`${label} must be between 1 and 65535`);
  }
  return port;
}

function isValidIpv4Address(value: string): boolean {
  const parts = value.split(".");
  return parts.length === 4 && parts.every((part) => /^\d+$/.test(part) && Number(part) >= 0 && Number(part) <= 255);
}

function isValidIpv6Address(value: string): boolean {
  try {
    new URL(`http://[${value.trim()}]/`);
    return true;
  } catch {
    return false;
  }
}

function isValidEndpointHostname(value: string): boolean {
  const normalized = value.trim().replace(/\.$/, "");
  if (!normalized.includes(".")) {
    return false;
  }
  if (!/[a-z]/i.test(normalized)) {
    return false;
  }
  return normalized.split(".").every((label) =>
    label.length > 0 &&
    label.length <= 63 &&
    !label.startsWith("-") &&
    !label.endsWith("-") &&
    /^[A-Za-z0-9-]+$/.test(label)
  );
}

function isDn42TunnelIpv4(value: string): boolean {
  if (!isValidIpv4Address(value)) {
    return false;
  }
  const [first, second] = value.split(".").map(Number);
  return first === 172 && second >= 20 && second <= 23;
}

function isUlaIpv6Address(value: string): boolean {
  const trimmed = value.trim().toLowerCase();
  return isValidIpv6Address(trimmed) && (trimmed.startsWith("fc") || trimmed.startsWith("fd"));
}

function isLinkLocalIpv6Address(value: string): boolean {
  const trimmed = value.trim().toLowerCase();
  return isValidIpv6Address(trimmed) && trimmed.startsWith("fe80:");
}

function validateWireGuardPublicKey(value: string): void {
  const trimmed = value.trim();
  if (trimmed.length !== 44 || !trimmed.endsWith("=") || !BASE64_LIKE.test(trimmed)) {
    throw new Error("wg_public_key must be a 44-character base64 public key");
  }
}

function validateWireGuardPsk(value: string): void {
  const trimmed = value.trim();
  if (trimmed.length !== 44 || !trimmed.endsWith("=") || !BASE64_LIKE.test(trimmed)) {
    throw new Error("psk must be a 44-character base64 WireGuard pre-shared key");
  }
}

function validatePeerIpv4(value: string): void {
  if (!isValidIpv4Address(value)) {
    throw new Error("peer4 must be a valid IPv4 address");
  }
  if (!isDn42TunnelIpv4(value)) {
    throw new Error("peer4 must be within 172.20.0.0/14");
  }
}

function validatePeerIpv6(value: string): void {
  if (!isValidIpv6Address(value)) {
    throw new Error("peer6 must be a valid IPv6 address");
  }
  if (!isLinkLocalIpv6Address(value) && !isUlaIpv6Address(value)) {
    throw new Error("peer6 must be a ULA or link-local IPv6 address");
  }
}

function validateMtu(value: number | null | undefined): void {
  if (value === null || value === undefined) {
    return;
  }
  if (value < 1280 || value > 1500) {
    throw new Error("mtu must be between 1280 and 1500");
  }
}

export function validateSessionSpec(node: InventoryHost, asn: string, spec: PeerSessionSpec): void {
  const peer4 = spec.peer4?.trim();
  const peer6 = spec.peer6?.trim();
  if (
    spec.mp_bgp_transport !== null &&
    spec.mp_bgp_transport !== undefined &&
    !isMpBgpTransport(spec.mp_bgp_transport)
  ) {
    throw new Error(`mp_bgp_transport must be one of ${MP_BGP_TRANSPORTS.join(", ")}`);
  }
  const transport = resolveMpBgpTransport(spec.mp_bgp_transport ?? null, peer4, peer6);
  parseEndpoint(spec.endpoint);
  if (!spec.wg_public_key.trim()) {
    throw new Error("wg_public_key is required");
  }
  validateWireGuardPublicKey(spec.wg_public_key);
  if (!peer4 && !peer6) {
    throw new Error("add at least one tunnel address: IPv4 or IPv6");
  }
  if (peer4) {
    validatePeerIpv4(peer4);
  }
  if (peer6) {
    validatePeerIpv6(peer6);
  }
  if (!spec.ipv4 && !spec.ipv6) {
    throw new Error("at least one BGP family must be enabled");
  }
  const peer4RequiredForTransport = spec.mp_bgp && transport === "ipv4";
  const peer6RequiredForTransport = spec.mp_bgp && transport === "ipv6";
  const peer4RequiredForRoutes = spec.ipv4 && !spec.mp_bgp;
  const peer6RequiredForRoutes = spec.ipv6 && !spec.mp_bgp;
  if (peer6RequiredForTransport && !peer6) {
    throw new Error("peer6 is required for MP-BGP over IPv6 transport");
  }
  if (peer4RequiredForTransport && !peer4) {
    throw new Error("peer4 is required for MP-BGP over IPv4 transport");
  }
  if (peer4RequiredForRoutes && !peer4) {
    throw new Error("peer4 is required for IPv4 routes");
  }
  if (peer6RequiredForRoutes && !peer6) {
    throw new Error("peer6 is required for IPv6 routes");
  }
  if (spec.extended_next_hop && !spec.mp_bgp) {
    throw new Error("extended_next_hop requires MP-BGP");
  }
  if (spec.extended_next_hop && !spec.ipv4) {
    throw new Error("extended_next_hop requires IPv4 routes");
  }
  if (spec.extended_next_hop && transport !== "ipv6") {
    throw new Error("extended_next_hop requires IPv6 transport");
  }
  if (spec.ipv4 && spec.mp_bgp && transport === "ipv6" && !peer4 && !spec.extended_next_hop) {
    throw new Error("ipv4 over IPv6 transport requires peer4 or extended_next_hop");
  }
  if (!isPeeringStrategy(spec.peering_strategy)) {
    throw new Error(`peering_strategy must be one of ${PEERING_STRATEGIES.join(", ")}`);
  }
  if (spec.own6?.trim()) {
    if (!peer6) {
      throw new Error("own6 requires an IPv6 tunnel address");
    }
    if (!isValidIpv6Address(spec.own6)) {
      throw new Error("own6 must be a valid IPv6 address");
    }
    if (!isLinkLocalIpv6Address(peer6)) {
      throw new Error("own6 only applies to link-local IPv6 peering");
    }
    if (!isLinkLocalIpv6Address(spec.own6)) {
      throw new Error("own6 must be a link-local IPv6 address");
    }
  }
  if (spec.port !== null && spec.port !== undefined && (spec.port < 1 || spec.port > 65535)) {
    throw new Error("port must be between 1 and 65535");
  }
  validateMtu(spec.mtu);
  if (spec.psk !== null && spec.psk !== undefined) {
    validateWireGuardPsk(spec.psk);
  }
  ensureEndpointAllowed(node, spec.endpoint);
  normalizeAsn(asn);
}

function resolveMpBgpTransport(
  explicit: MpBgpTransport | null,
  peer4: string | undefined,
  peer6: string | undefined,
): MpBgpTransport | null {
  if (explicit) {
    return explicit;
  }
  if (peer6) {
    return "ipv6";
  }
  if (peer4) {
    return "ipv4";
  }
  return null;
}

export function loadInventoryHosts(
  inventoryText: string,
  autopeerPolicyText?: string | null,
): InventoryHost[] {
  const inventory = toPlainObject(inventoryText);
  const all = inventory.all;
  if (!isTruthyRecord(all)) {
    throw new Error("inventory.yaml is missing all");
  }
  const children = all.children;
  if (!isTruthyRecord(children)) {
    throw new Error("inventory.yaml is missing all.children");
  }
  const nodeHosts = isTruthyRecord(children.nodes) && isTruthyRecord(children.nodes.hosts)
    ? (children.nodes.hosts as Record<string, unknown>)
    : null;
  const dn42Hosts = isTruthyRecord(children.dn42) && isTruthyRecord(children.dn42.hosts)
    ? (children.dn42.hosts as Record<string, unknown>)
    : null;

  if (!nodeHosts || !dn42Hosts) {
    throw new Error("inventory.yaml must define nodes.hosts and dn42.hosts");
  }

  let enabledNames: Set<string> | null = null;
  if (autopeerPolicyText) {
    const policy = toPlainObject(autopeerPolicyText);
    const autopeer = isTruthyRecord(policy.autopeer) ? policy.autopeer : policy;
    const rawNodes =
      (Array.isArray(autopeer.enabled_nodes) ? autopeer.enabled_nodes : null) ??
      (Array.isArray(autopeer.nodes) ? autopeer.nodes : null);
    if (rawNodes) {
      enabledNames = new Set(rawNodes.map((value) => String(value)));
    }
  }

  return Object.entries(dn42Hosts)
    .filter(([name]) => !enabledNames || enabledNames.has(name))
    .map(([name]) => {
      const source = isTruthyRecord(nodeHosts[name]) ? (nodeHosts[name] as Record<string, unknown>) : {};
      return {
        name,
        endpoint_host: typeof source.ansible_host === "string" ? source.ansible_host : undefined,
        region: typeof source.region === "string" ? source.region : undefined,
        country: typeof source.country === "string" ? source.country : undefined,
        ip_support: typeof source.ip_support === "string" ? source.ip_support : "dual",
        comment: typeof source.peering_comment === "string" ? source.peering_comment : undefined,
        peering: {
          ipv4: typeof source.ownip === "string" ? source.ownip : undefined,
          ipv6: typeof source.ownip6 === "string" ? source.ownip6 : undefined,
          link_local_ipv6:
            typeof source.link_local_ipv6 === "string" ? source.link_local_ipv6 : undefined,
          wg_pubkey: typeof source.wg_pubkey === "string" ? source.wg_pubkey : undefined,
          comment: typeof source.peering_comment === "string" ? source.peering_comment : undefined,
        },
        autopeer: source.autopeer === false ? false : undefined,
      };
    })
    .sort((left, right) => left.name.localeCompare(right.name));
}

export function buildNodeViews(hosts: InventoryHost[]): NodeView[] {
  return hosts.map((host) => ({
    name: host.name,
    endpoint_host: host.endpoint_host,
    region: host.region,
    country: host.country,
    ip_support: host.ip_support,
    comment: host.comment,
    peering: host.peering,
    autopeer: host.autopeer,
  }));
}

export async function listSessionsForAsn(
  asn: string,
  peerFiles: Map<string, string>,
  hosts: InventoryHost[],
  operations: OperationRecord[],
  vaultPassword: string | null,
): Promise<SessionView[]> {
  const sessions = new Map<string, SessionView>();

  const matchingPeers: { host: InventoryHost; peer: PeerEntry }[] = [];
  for (const host of hosts) {
    const text = peerFiles.get(host.name);
    if (!text) continue;
    const data = normalizePeerFileData(parseYamlObject(text));
    for (const peer of data.peers) {
      if (peerAsn(peer) !== asn || isRemovedPeer(peer)) continue;
      matchingPeers.push({ host, peer });
    }
  }

  const specs = await Promise.all(matchingPeers.map(({ peer }) => toSpec(peer, vaultPassword)));

  for (let i = 0; i < matchingPeers.length; i++) {
    const { host, peer } = matchingPeers[i];
    sessions.set(host.name, {
      node: host.name,
      asn,
      state: isManagedPeer(peer) ? "managed" : "manual",
      spec: specs[i],
      metadata: isManagedPeer(peer)
        ? {
            managed: true,
            effective_mnt:
              isTruthyRecord(peer.autopeer) && typeof peer.autopeer.effective_mnt === "string"
                ? String(peer.autopeer.effective_mnt)
                : undefined,
            auth_provider:
              isTruthyRecord(peer.autopeer) && typeof peer.autopeer.auth_provider === "string"
                ? String(peer.autopeer.auth_provider)
                : undefined,
          }
        : { managed: false },
      has_psk: peerHasPsk(peer) || undefined,
      has_encrypted_endpoint: peerHasEncryptedEndpoint(peer) || undefined,
    });
  }

  for (const operation of operations) {
    const current = sessions.get(operation.node);
    const isPending = !["completed", "failed", "conflict"].includes(operation.state);
    if (!isPending) {
      continue;
    }

    sessions.set(operation.node, {
      node: operation.node,
      asn,
      state: "pending_pr",
      spec: operation.session_snapshot ?? current?.spec,
      metadata: current?.metadata ?? { managed: true },
      has_psk: current?.has_psk,
      has_encrypted_endpoint: current?.has_encrypted_endpoint,
      pending_operation_id: operation.id,
      pull_request_url: operation.pull_request_url ?? undefined,
      message: operation.message ?? undefined,
    });
  }

  return [...sessions.values()].sort((left, right) => left.node.localeCompare(right.node));
}

export function nodeIsAvailable(node: InventoryHost, sessions: SessionView[]): boolean {
  return !sessions.some((session) => session.node === node.name && session.state !== "manual");
}

async function vaultProtectEntry(
  entry: PeerEntry,
  spec: PeerSessionSpec | undefined,
  existing: PeerEntry | undefined,
  vaultPassword: string | null,
): Promise<void> {
  const wg = entry.wg as YamlObject | undefined;
  if (!wg || !vaultPassword) return;

  if (typeof wg.psk === "string" && wg.psk.length > 0) {
    wg.psk = await vaultEncrypt(wg.psk, vaultPassword);
  }

  const shouldEncryptEndpoint =
    spec?.encrypt_endpoint === true ||
    (spec?.encrypt_endpoint === undefined &&
      existing &&
      peerHasEncryptedEndpoint(existing));

  if (shouldEncryptEndpoint && typeof wg.endpoint === "string") {
    wg.endpoint = await vaultEncrypt(wg.endpoint, vaultPassword);
  }
}

function sanitizeSnapshot(spec: PeerSessionSpec | null): PeerSessionSpec | null {
  if (!spec) return null;
  const { psk: _, ...rest } = spec;
  return rest;
}

async function finalizeEntry(
  next: PeerEntry,
  spec: PeerSessionSpec | undefined,
  existing: PeerEntry | undefined,
  vaultPassword: string | null,
): Promise<PeerSessionSpec | null> {
  const snapshot = sanitizeSnapshot(await toSpec(next));
  await vaultProtectEntry(next, spec, existing, vaultPassword);
  return snapshot;
}

export async function mutatePeerFile(
  fileText: string,
  input: {
    asn: string;
    effectiveMnt: string;
    authMethod: AuthMethod;
    kind: OperationKind;
    session?: PeerSessionSpec;
    vaultPassword: string | null;
  },
): Promise<{ content: string; sessionSnapshot: PeerSessionSpec | null }> {
  const data = parseYamlObject(fileText);
  const normalized = normalizePeerFileData(data);
  const peers = [...normalized.peers];
  const matchIndexes = peers
    .map((peer, index) => ({ peer, index }))
    .filter(({ peer }) => peerAsn(peer) === input.asn);

  if (matchIndexes.length > 1) {
    throw new Error(`duplicate ASN ${input.asn} exists in peer file`);
  }

  const match = matchIndexes[0];
  const existing = match?.peer;
  const isManaged = existing ? isManagedPeer(existing) : false;
  const isRemoved = existing ? isRemovedPeer(existing) : false;

  if (input.kind === "create") {
    if (existing && !isRemoved) {
      if (!input.session) {
        throw new Error("create requires a session payload");
      }
      if (!isManaged) {
        const next = specToPeerEntry(
          input.asn,
          input.effectiveMnt,
          migrationAuthMethod(input.authMethod),
          input.session,
          existing,
        );
        const sessionSnapshot = await finalizeEntry(next, input.session, existing, input.vaultPassword);
        peers[match.index] = next;
        return {
          content: dumpPeerYaml(normalizePeerFileData({ peers })),
          sessionSnapshot,
        };
      }
      throw new Error(`managed peer AS${input.asn} already exists on this node`);
    }

    if (!input.session) {
      throw new Error("create requires a session payload");
    }

    const next = specToPeerEntry(
      input.asn,
      input.effectiveMnt,
      input.authMethod,
      input.session,
      existing,
    );
    const sessionSnapshot = await finalizeEntry(next, input.session, existing, input.vaultPassword);
    if (match) {
      peers[match.index] = next;
    } else {
      peers.push(next);
    }
    return {
      content: dumpPeerYaml(normalizePeerFileData({ peers })),
      sessionSnapshot,
    };
  }

  if (!existing) {
    throw new Error(`peer AS${input.asn} does not exist on this node`);
  }
  if (input.kind === "migrate") {
    if (isManaged) {
      throw new Error(`peer AS${input.asn} is already managed by autopeer`);
    }
    const next = specToPeerEntry(
      input.asn,
      input.effectiveMnt,
      input.authMethod,
      await toSpec(existing, input.vaultPassword),
      existing,
    );
    const sessionSnapshot = await finalizeEntry(next, undefined, existing, input.vaultPassword);
    peers[match.index] = next;
    return {
      content: dumpPeerYaml(normalizePeerFileData({ peers })),
      sessionSnapshot,
    };
  }

  if (input.kind === "update") {
    if (!input.session) {
      throw new Error("update requires a session payload");
    }
    const next = specToPeerEntry(
      input.asn,
      input.effectiveMnt,
      isManaged ? input.authMethod : migrationAuthMethod(input.authMethod),
      input.session,
      existing,
    );
    const sessionSnapshot = await finalizeEntry(next, input.session, existing, input.vaultPassword);
    peers[match.index] = next;
    return {
      content: dumpPeerYaml(normalizePeerFileData({ peers })),
      sessionSnapshot,
    };
  }

  if (!isManaged) {
    throw new Error(`manual peer AS${input.asn} cannot be modified by autopeer`);
  }

  const removedEntry = normalizePeerEntry({
    ...existing,
    removed: true,
  });
  peers[match.index] = removedEntry;
  return {
    content: dumpPeerYaml(normalizePeerFileData({ peers })),
    sessionSnapshot: sanitizeSnapshot(existing ? await toSpec(existing, input.vaultPassword) : null),
  };
}
