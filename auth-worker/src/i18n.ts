import type { TranslatorFn } from "dn42-auth-worker/mailer";

export type WorkerLocale = "en" | "de" | "zh" | "la";

export function resolveLocaleCode(code: string | null | undefined): WorkerLocale | undefined {
  if (!code) return undefined;
  const primary = code.split(/[-_,;]/)[0]?.trim().toLowerCase();
  if (!primary) return undefined;
  if (primary === "de") return "de";
  if (primary === "zh") return "zh";
  if (primary === "la") return "la";
  if (primary === "en") return "en";
  return undefined;
}

export function resolveLocale(request: Request): WorkerLocale {
  return resolveLocaleCode(request.headers.get("Accept-Language")) ?? "en";
}

const tables: Record<WorkerLocale, Record<string, string>> = {
  en: {
    "email.subject": "dn42 Auth login for AS{asn}",
    "email.intro_html":
      'Use this sign-in link or one-time code to sign in to dn42 Auth for <strong>AS{asn}</strong> as <strong>{mnt}</strong>.',
    "email.intro_text":
      "Use this sign-in link or one-time code to sign in to dn42 Auth for AS{asn} as {mnt}.",
    "email.link_label": "Open Sign-In Link",
    "email.code_intro": "Your one-time auth code is:",
    "email.expires": "This code expires at {expires_at}.",
    "email.ignore": "If you did not start this login, you can ignore this email.",
  },
  de: {
    "email.subject": "dn42 Auth-Login für AS{asn}",
    "email.intro_html":
      'Verwenden Sie diesen Anmeldelink oder Einmalcode, um sich bei dn42 Auth für <strong>AS{asn}</strong> als <strong>{mnt}</strong> anzumelden.',
    "email.intro_text":
      "Verwenden Sie diesen Anmeldelink oder Einmalcode, um sich bei dn42 Auth für AS{asn} als {mnt} anzumelden.",
    "email.link_label": "Anmeldelink öffnen",
    "email.code_intro": "Ihr Einmal-Auth-Code lautet:",
    "email.expires": "Dieser Code läuft um {expires_at} ab.",
    "email.ignore": "Wenn Sie diesen Login nicht gestartet haben, können Sie diese E-Mail ignorieren.",
  },
  zh: {
    "email.subject": "dn42 Auth AS{asn} 登录验证",
    "email.intro_html":
      '请使用以下登录链接或一次性验证码，以 <strong>{mnt}</strong> 身份登录 dn42 Auth（<strong>AS{asn}</strong>）。',
    "email.intro_text":
      "请使用以下登录链接或一次性验证码，以 {mnt} 身份登录 dn42 Auth（AS{asn}）。",
    "email.link_label": "打开登录链接",
    "email.code_intro": "您的一次性验证码为：",
    "email.expires": "此验证码将于 {expires_at} 过期。",
    "email.ignore": "如果您没有发起此登录请求，请忽略此邮件。",
  },
  la: {
    "email.subject": "dn42 Auth aditus pro AS{asn}",
    "email.intro_html":
      'Utere hoc nexu aut codice unico ut dn42 Auth pro <strong>AS{asn}</strong> tamquam <strong>{mnt}</strong> intres.',
    "email.intro_text":
      "Utere hoc nexu aut codice unico ut dn42 Auth pro AS{asn} tamquam {mnt} intres.",
    "email.link_label": "Aperi Nexum Aditus",
    "email.code_intro": "Codex unicus tuus est:",
    "email.expires": "Hic codex exspirat ad {expires_at}.",
    "email.ignore": "Si hunc aditum non incepisti, hanc epistulam ignorare potes.",
  },
};

function t(locale: WorkerLocale, key: string, params?: Record<string, string>): string {
  let text = tables[locale][key] ?? tables.en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replaceAll(`{${k}}`, v);
    }
  }
  return text;
}

export function translator(locale: WorkerLocale): TranslatorFn {
  return (key, params) => t(locale, key, params);
}
