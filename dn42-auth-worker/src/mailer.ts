import type { RegistryEmailAuthRequestRecord } from "./types";
import { HttpError, readSecret, uiMessage } from "./utils";

const RESEND_EMAILS_ENDPOINT = "https://api.resend.com/emails";
const AUTOPEER_FROM = "IRIS-AS Autopeer <autopeer@owo.li>";

export type TranslatorFn = (key: string, params?: Record<string, string>) => string;

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function emailHtml(
  t: TranslatorFn,
  asn: string,
  effectiveMnt: string,
  request: RegistryEmailAuthRequestRecord,
  magicLinkUrl: string,
): string {
  const escapedLink = escapeHtml(magicLinkUrl);
  const escapedCode = escapeHtml(request.code);
  const params = {
    asn: escapeHtml(asn),
    mnt: escapeHtml(effectiveMnt),
    expires_at: escapeHtml(request.expires_at),
  };

  return [
    "<div style=\"font-family:system-ui,-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;line-height:1.5;color:#111827\">",
    `<p>${t("email.intro_html", params)}</p>`,
    `<p><a href="${escapedLink}">${escapeHtml(t("email.link_label"))}</a></p>`,
    `<p>${escapeHtml(t("email.code_intro"))}</p>`,
    `<p style="font-size:1.5rem;font-weight:700;letter-spacing:0.18em">${escapedCode}</p>`,
    `<p>${t("email.expires", { expires_at: escapeHtml(request.expires_at) })}</p>`,
    `<p>${escapeHtml(t("email.ignore"))}</p>`,
    "</div>",
  ].join("");
}

function emailText(
  t: TranslatorFn,
  asn: string,
  effectiveMnt: string,
  request: RegistryEmailAuthRequestRecord,
  magicLinkUrl: string,
): string {
  const params = { asn, mnt: effectiveMnt };
  return [
    t("email.intro_text", params),
    "",
    `${t("email.link_label")}: ${magicLinkUrl}`,
    "",
    `${t("email.code_intro")} ${request.code}`,
    t("email.expires", { expires_at: request.expires_at }),
    "",
    t("email.ignore"),
  ].join("\n");
}

export async function sendRegistryEmailAuthMessage(
  env: object,
  t: TranslatorFn,
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
      subject: t("email.subject", { asn }),
      html: emailHtml(t, asn, effectiveMnt, request, magicLinkUrl),
      text: emailText(t, asn, effectiveMnt, request, magicLinkUrl),
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

  throw new HttpError(uiMessage("error.email.send_failed", { detail }), 502);
}
