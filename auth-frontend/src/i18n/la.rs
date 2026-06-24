pub(super) const TABLE: &[(&str, &str)] = &[
    ("app.title", "dn42 Auth"),
    ("app.subtitle", "ex IRIS-AS 4242421023"),
    (
        "auth.prompt",
        "Inscribe ASN tuum dn42 pro auctoritate SSH, PGP, vel electronica.",
    ),
    (
        "auth.asn_methods",
        "Methodos auctoritatis pro AS{asn} invenimus",
    ),
    ("auth.email_verify", "Verificatio electronica pro AS{asn}"),
    ("auth.authenticated", "Authenticatus ut AS{asn} ({mnt})"),
    ("auth.redirecting", "Redirectio\u{2026}"),
    ("auth.loading", "Onerans\u{2026}"),
    (
        "auth.oidc_alt",
        "Vel intra per provisorem identitatis et ASN tuum automatice deducemus.",
    ),
    ("auth.continue_with", "Perge cum {provider}"),
    (
        "auth.email_intro",
        "Mitte nexum intrandi et codicem unicum ad inscriptiones electronicas unius ex curatoribus tuis, deinde nexum preme aut codicem infra adglutina.",
    ),
    ("auth.email_auth_as", "Authenticare ut {mnt}"),
    ("auth.email_send_to", "Mittere ad {emails}"),
    (
        "auth.email_code_prompt",
        "Adglutina codicem ex epistula tua",
    ),
    (
        "auth.code_sent",
        "Nexum intrandi et codicem ad {emails} misimus.",
    ),
    ("prompt.emails", "emails"),
    ("action.find_methods", "Quaerere methodos auctoritatis"),
    ("action.verify", "Verificare"),
    ("action.verify_code", "Verificare codicem"),
    ("action.send_code", "Mittere nexum intrandi"),
    ("action.resend_code", "Remittere nexum intrandi"),
    ("action.back", "Retro"),
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
        "Mandatum supra exsequere, deinde signaturam SSH tuam adglutina.",
    ),
    (
        "pgp.paste_signed",
        "Adglutina nuntium tuum plene clearsign.",
    ),
    (
        "pgp.paste_pubkey",
        "Adglutina clavem publicam tuam ASCII-armatam.",
    ),
    ("nav.language", "Lingua"),
    (
        "auth.oidc_redirecting",
        "Redirectio ad provisorem OIDC tuum\u{2026}",
    ),
    ("error.auth.return_to.invalid", "Destinatio invalida."),
    ("auth_method.registry_ssh.label", "Subscriptio SSH Registri"),
    ("auth_method.registry_pgp.label", "Subscriptio PGP Registri"),
    ("auth_method.registry_email.label", "Electronica Registri"),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
