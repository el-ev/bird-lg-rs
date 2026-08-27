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
    ("auth.complete_close", "Hanc paginam nunc claudere potes."),
    ("auth.loading", "Onerans\u{2026}"),
    (
        "auth.finding_methods",
        "Methodos auctoritatis pro ASN tuo quaerimus\u{2026}",
    ),
    (
        "auth.oidc_alt",
        "Vel intra per provisorem identitatis et ASN tuum automatice reperiemus.",
    ),
    ("auth.continue_with", "Perge cum {provider}"),
    (
        "auth.email_intro",
        "Elige curatorem et nexum intrandi cum codice unico ad inscriptiones electronicas registri eius mittemus. Deinde nexum ex epistula aperi, aut codicem infra adglutina.",
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
    ("block.challenge", "provocatio"),
    ("block.sign_command", "mandatum subscribendi"),
    (
        "block.export_command",
        "mandatum clavis publicae exportandae",
    ),
    ("nav.language", "Lingua"),
    (
        "auth.oidc_redirecting",
        "Redirectio ad provisorem OIDC tuum\u{2026}",
    ),
    ("error.auth.return_to.invalid", "Destinatio invalida."),
    (
        "error.registry.lookup_failed",
        "Quaestio AS{asn} in registro dn42 defecit. Postea iterum conare.",
    ),
    (
        "error.registry.unavailable",
        "Registrum dn42 legi non potuit (causa: {reason}). Vitium apud ministerium authenticationis est, non apud AS{asn}. Postea iterum conare aut operatorem appella.",
    ),
    ("auth_method.registry_ssh.label", "Subscriptio SSH"),
    ("auth_method.registry_pgp.label", "Subscriptio PGP"),
    ("auth_method.registry_email.label", "Electronica"),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
