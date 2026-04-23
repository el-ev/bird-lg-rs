export type WorkerLocale = "en" | "zh" | "la";

export function resolveLocale(request: Request): WorkerLocale {
  const header = request.headers.get("Accept-Language");
  if (!header) return "en";
  const primary = header.split(/[-_,;]/)[0].trim().toLowerCase();
  if (primary === "zh") return "zh";
  if (primary === "la") return "la";
  return "en";
}

const tables: Record<WorkerLocale, Record<string, string>> = {
  en: {
    "email.subject": "dn42 Autopeer login for AS{asn}",
    "email.intro_html":
      'Use this sign-in link or one-time code to sign in to dn42 Autopeer for <strong>AS{asn}</strong> as <strong>{mnt}</strong>.',
    "email.intro_text":
      "Use this sign-in link or one-time code to sign in to dn42 Autopeer for AS{asn} as {mnt}.",
    "email.link_label": "Open Autopeer Sign-In Link",
    "email.code_intro": "Your one-time auth code is:",
    "email.expires": "This code expires at {expires_at}.",
    "email.ignore": "If you did not start this login, you can ignore this email.",
    "pr.body": "Autopeer {kind} request for AS{asn}.",
    "pr.node": "Node",
    "pr.maintainer": "Maintainer",
    "pr.auth": "Auth",
    "kind.create": "create",
    "kind.update": "update",
    "kind.delete": "retire",
    "kind.purge": "delete",
    "kind.migrate": "migrate",
  },
  zh: {
    "email.subject": "dn42 自动对等互联 AS{asn} 登录验证",
    "email.intro_html":
      '请使用以下登录链接或一次性验证码，以 <strong>{mnt}</strong> 身份登录 dn42 自动对等互联（<strong>AS{asn}</strong>）。',
    "email.intro_text":
      "请使用以下登录链接或一次性验证码，以 {mnt} 身份登录 dn42 自动对等互联（AS{asn}）。",
    "email.link_label": "打开自动对等互联登录链接",
    "email.code_intro": "您的一次性验证码为：",
    "email.expires": "此验证码将于 {expires_at} 过期。",
    "email.ignore": "如果您没有发起此登录请求，请忽略此邮件。",
    "pr.body": "来自 AS{asn} 的自动对等互联{kind}请求。",
    "pr.node": "节点",
    "pr.maintainer": "维护者",
    "pr.auth": "认证方式",
    "kind.create": "创建",
    "kind.update": "更新",
    "kind.delete": "停用",
    "kind.purge": "删除",
    "kind.migrate": "迁移",
  },
  la: {
    "email.subject": "dn42 Autopeer aditus pro AS{asn}",
    "email.intro_html":
      'Utere hoc nexu aut codice unico ut dn42 Autopeer pro <strong>AS{asn}</strong> tamquam <strong>{mnt}</strong> intres.',
    "email.intro_text":
      "Utere hoc nexu aut codice unico ut dn42 Autopeer pro AS{asn} tamquam {mnt} intres.",
    "email.link_label": "Aperi Nexum Aditus Autopeer",
    "email.code_intro": "Codex unicus tuus est:",
    "email.expires": "Hic codex exspirat ad {expires_at}.",
    "email.ignore": "Si hunc aditum non incepisti, hanc epistulam ignorare potes.",
    "pr.body": "Autopeer petitio {kind} pro AS{asn}.",
    "pr.node": "Nodus",
    "pr.maintainer": "Curator",
    "pr.auth": "Auctoritas",
    "kind.create": "creandi",
    "kind.update": "renovandi",
    "kind.delete": "retrahendi",
    "kind.purge": "delendi",
    "kind.migrate": "migrandi",
  },
};

export function t(
  locale: WorkerLocale,
  key: string,
  params?: Record<string, string>,
): string {
  let text = tables[locale][key] ?? tables.en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replaceAll(`{${k}}`, v);
    }
  }
  return text;
}
