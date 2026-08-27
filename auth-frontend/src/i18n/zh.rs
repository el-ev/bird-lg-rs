pub(super) const TABLE: &[(&str, &str)] = &[
    ("app.title", "dn42 Auth"),
    ("app.subtitle", "IRIS-AS 4242421023"),
    (
        "auth.prompt",
        "输入你的 dn42 ASN 以使用注册库 SSH、PGP 或邮件认证。",
    ),
    ("auth.asn_methods", "我们找到了 AS{asn} 的注册库认证方法"),
    ("auth.email_verify", "AS{asn} 的邮件验证"),
    ("auth.authenticated", "已认证为 AS{asn} ({mnt})"),
    ("auth.redirecting", "正在跳转\u{2026}"),
    ("auth.complete_close", "你现在可以关闭此页面了。"),
    ("auth.loading", "加载中\u{2026}"),
    (
        "auth.finding_methods",
        "正在查找你的 ASN 的认证方法\u{2026}",
    ),
    (
        "auth.oidc_alt",
        "或使用身份提供商登录，我们将自动推断你的 ASN。",
    ),
    ("auth.continue_with", "使用 {provider} 继续"),
    (
        "auth.email_intro",
        "将登录链接和一次性验证码发送到你的某个维护者的注册库邮箱联系人，然后点击链接或在下方粘贴验证码。",
    ),
    ("auth.email_auth_as", "以 {mnt} 身份认证"),
    ("auth.email_send_to", "发送至 {emails}"),
    ("auth.email_code_prompt", "粘贴你邮件中的验证码"),
    (
        "auth.code_sent",
        "我们已向 {emails} 发送了登录链接和验证码。",
    ),
    ("prompt.emails", "emails"),
    ("action.find_methods", "查找注册库认证方法"),
    ("action.verify", "验证"),
    ("action.verify_code", "验证代码"),
    ("action.send_code", "发送登录链接"),
    ("action.resend_code", "重新发送登录链接"),
    ("action.back", "返回"),
    ("prompt.auth", "auth"),
    ("prompt.asn", "ASN"),
    ("prompt.key", "key"),
    ("prompt.signature", "signature"),
    ("prompt.signed", "signed"),
    ("prompt.pubkey", "pubkey"),
    ("prompt.mntner", "mntner"),
    ("prompt.code", "code"),
    ("prompt.status", "status"),
    (
        "ssh.paste_prompt",
        "运行上面的命令，然后粘贴你的 SSH 签名。",
    ),
    ("pgp.paste_signed", "粘贴你的完整 clearsign 消息。"),
    ("pgp.paste_pubkey", "粘贴你的 ASCII 格式公钥。"),
    ("block.challenge", "挑战文本"),
    ("block.sign_command", "签名命令"),
    ("block.export_command", "公钥导出命令"),
    ("nav.language", "语言"),
    (
        "auth.oidc_redirecting",
        "正在跳转到你的 OIDC 提供商\u{2026}",
    ),
    ("error.auth.return_to.invalid", "无效的返回地址。"),
    (
        "error.registry.lookup_failed",
        "在 dn42 注册库中查询 AS{asn} 失败，请稍后重试。",
    ),
    (
        "error.registry.unavailable",
        "当前无法读取 dn42 注册库（原因：{reason}）。这是认证服务侧的问题，与 AS{asn} 无关，请稍后重试或联系运营者。",
    ),
    ("auth_method.registry_ssh.label", "SSH 签名"),
    ("auth_method.registry_pgp.label", "PGP 签名"),
    ("auth_method.registry_email.label", "邮件"),
    ("auth_method.host_impersonation.label", "主 ASN 冒充"),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
