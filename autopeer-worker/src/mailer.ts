import type { RegistryEmailAuthRequestRecord } from "./types";
import { readSecret } from "./utils";

const RESEND_EMAILS_ENDPOINT = "https://api.resend.com/emails";
const AUTOPEER_FROM = "IRIS-AS Autopeer <autopeer@owo.li>";

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function emailHtml(
  asn: string,
  effectiveMnt: string,
  request: RegistryEmailAuthRequestRecord,
  magicLinkUrl: string,
): string {
  const escapedLink = escapeHtml(magicLinkUrl);
  const escapedMaintainer = escapeHtml(effectiveMnt);
  const escapedCode = escapeHtml(request.code);
  const escapedAsn = escapeHtml(asn);
  const escapedExpiry = escapeHtml(request.expires_at);

  return [
    "<div style=\"font-family:system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;line-height:1.5;color:#111827\">",
    `<p>Use this sign-in link or one-time code to sign in to DN42 Autopeer for <strong>AS${escapedAsn}</strong> as <strong>${escapedMaintainer}</strong>.</p>`,
    `<p><a href="${escapedLink}">Open Autopeer Sign-In Link</a></p>`,
    `<p>Your one-time auth code is:</p>`,
    `<p style="font-size:1.5rem;font-weight:700;letter-spacing:0.18em">${escapedCode}</p>`,
    `<p>This code expires at ${escapedExpiry}.</p>`,
    "<p>If you did not start this login, you can ignore this email.</p>",
    "</div>",
  ].join("");
}

function emailText(
  asn: string,
  effectiveMnt: string,
  request: RegistryEmailAuthRequestRecord,
  magicLinkUrl: string,
): string {
  return [
    `Use this sign-in link or one-time code to sign in to DN42 Autopeer for AS${asn} as ${effectiveMnt}.`,
    "",
    `Sign-in link: ${magicLinkUrl}`,
    "",
    `One-time auth code: ${request.code}`,
    `Expires at: ${request.expires_at}`,
    "",
    "If you did not start this login, you can ignore this email.",
  ].join("\n");
}

export async function sendRegistryEmailAuthMessage(
  env: Env,
  asn: string,
  effectiveMnt: string,
  request: RegistryEmailAuthRequestRecord,
  magicLinkUrl: string,
): Promise<void> {
  const response = await fetch(RESEND_EMAILS_ENDPOINT, {
    method: "POST",
    headers: {
      authorization: `Bearer ${readSecret(env, "RESEND_API_KEY")}`,
      "content-type": "application/json",
      "idempotency-key": `registry-email/${request.challenge_id}/${request.created_at}`,
    },
    body: JSON.stringify({
      from: AUTOPEER_FROM,
      to: request.email_snapshot,
      subject: `DN42 Autopeer login for AS${asn}`,
      html: emailHtml(asn, effectiveMnt, request, magicLinkUrl),
      text: emailText(asn, effectiveMnt, request, magicLinkUrl),
    }),
  });

  if (response.ok) {
    return;
  }

  let detail = `HTTP ${response.status}`;
  try {
    const body = await response.json() as { message?: string; error?: string };
    detail = body.message ?? body.error ?? detail;
  } catch {
    // Keep the HTTP status fallback.
  }

  throw new Error(`Resend email send failed: ${detail}`);
}
