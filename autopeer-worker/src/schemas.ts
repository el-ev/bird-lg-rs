import { z } from "zod";
import {
  MP_BGP_TRANSPORTS,
  type MpBgpTransport,
  type PeerSessionSpec,
  type PeeringStrategy,
} from "./types";

const nonEmptyString = z.unknown().transform((value, ctx): string => {
  if (typeof value !== "string" || value.trim().length === 0) {
    ctx.addIssue({
      code: "custom",
      params: { uiKey: "error.field.required" },
    });
    return z.NEVER;
  }
  return value.trim();
});

const optionalTrimmedString = z.unknown().optional().transform((value, ctx): string | null => {
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "string") {
    ctx.addIssue({ code: "invalid_type", expected: "string", input: value });
    return z.NEVER;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
});

const optionalInteger = z.unknown().optional().transform((value, ctx): number | null => {
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "number" || !Number.isInteger(value)) {
    ctx.addIssue({ code: "invalid_type", expected: "int", input: value });
    return z.NEVER;
  }
  return value;
});

const PeerSessionSpecSchema = z.object({
  comment: optionalTrimmedString,
  endpoint: optionalTrimmedString,
  wg_public_key: nonEmptyString,
  port: optionalInteger,
  peer4: optionalTrimmedString,
  peer6: optionalTrimmedString,
  own6: optionalTrimmedString,
  keepalive: optionalInteger,
  mtu: optionalInteger,
  ipv4: z.boolean(),
  ipv6: z.boolean(),
  extended_next_hop: z.boolean(),
  mp_bgp: z.boolean(),
  mp_bgp_transport: z.unknown().optional().transform((value, ctx): MpBgpTransport | null => {
    if (value === null || value === undefined) {
      return null;
    }
    if (typeof value !== "string") {
      ctx.addIssue({ code: "invalid_type", expected: "string", input: value });
      return z.NEVER;
    }
    const trimmed = value.trim();
    if (trimmed.length === 0) {
      return null;
    }
    if (!(MP_BGP_TRANSPORTS as readonly string[]).includes(trimmed)) {
      ctx.addIssue({
        code: "custom",
        params: { uiKey: "error.request.session.mp_bgp_transport.invalid", literal: true },
      });
      return z.NEVER;
    }
    return trimmed as MpBgpTransport;
  }),
  peering_strategy: z.unknown().optional().transform((value, ctx): PeeringStrategy => {
    if (value === null || value === undefined) {
      return "full_table";
    }
    if (typeof value !== "string") {
      ctx.addIssue({ code: "invalid_type", expected: "string", input: value });
      return z.NEVER;
    }
    const trimmed = value.trim();
    return (trimmed.length > 0 ? trimmed : "full_table") as PeeringStrategy;
  }),
  psk: z.unknown().optional().transform((value, ctx): string | null | undefined => {
    if (value === undefined) {
      return undefined;
    }
    if (value === null) {
      return null;
    }
    if (typeof value !== "string") {
      ctx.addIssue({ code: "invalid_type", expected: "string", input: value });
      return z.NEVER;
    }
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
  }),
  encrypt_endpoint: z.boolean().optional(),
}) satisfies z.ZodType<PeerSessionSpec, unknown>;

export const CreateSessionSchema = z.object({
  node: nonEmptyString,
  session: PeerSessionSpecSchema,
});

export const UpdateSessionSchema = z.object({
  session: PeerSessionSpecSchema,
});

export type CreateSessionRequest = z.infer<typeof CreateSessionSchema>;
export type UpdateSessionRequest = z.infer<typeof UpdateSessionSchema>;
