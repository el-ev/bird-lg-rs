import { z } from "zod";

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

export const AuthStartSchema = z.object({
  asn: nonEmptyString,
});

export const HostImpersonationSchema = z.object({
  asn: nonEmptyString,
  effective_mnt: optionalTrimmedString,
});

export const RegistrySshVerifySchema = z.object({
  challenge_id: nonEmptyString,
  signature: nonEmptyString,
});

export const RegistryPgpVerifySchema = z.object({
  challenge_id: nonEmptyString,
  public_key: nonEmptyString,
  signed_message: nonEmptyString,
});

export const RegistryEmailSendSchema = z.object({
  challenge_id: nonEmptyString,
  effective_mnt: optionalTrimmedString,
  locale: optionalTrimmedString,
});

export const RegistryEmailVerifySchema = z.object({
  challenge_id: nonEmptyString,
  code: nonEmptyString,
});

export const RegistryEmailCompleteSchema = z.object({
  token: nonEmptyString,
});

export const OidcStartSchema = z.object({
  challenge_id: optionalTrimmedString,
  return_to: optionalTrimmedString,
});

export const OidcCompleteSchema = z.object({
  state: nonEmptyString,
});

export type AuthStartRequest = z.infer<typeof AuthStartSchema>;
export type HostImpersonationRequest = z.infer<typeof HostImpersonationSchema>;
export type RegistrySshVerifyRequest = z.infer<typeof RegistrySshVerifySchema>;
export type RegistryPgpVerifyRequest = z.infer<typeof RegistryPgpVerifySchema>;
export type RegistryEmailSendRequest = z.infer<typeof RegistryEmailSendSchema>;
export type RegistryEmailVerifyRequest = z.infer<typeof RegistryEmailVerifySchema>;
export type RegistryEmailCompleteRequest = z.infer<typeof RegistryEmailCompleteSchema>;
export type OidcStartRequest = z.infer<typeof OidcStartSchema>;
export type OidcCompleteRequest = z.infer<typeof OidcCompleteSchema>;
