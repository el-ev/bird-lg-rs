pub(super) const TABLE: &[(&str, &str)] = &[
    ("app.title", "dn42 Auth"),
    ("app.subtitle", "von IRIS-AS 4242421023"),
    (
        "auth.prompt",
        "Geben Sie Ihre dn42 ASN für SSH-, PGP- oder E-Mail-Authentifizierung ein.",
    ),
    (
        "auth.asn_methods",
        "Wir haben Authentifizierungsmethoden für AS{asn} gefunden",
    ),
    ("auth.email_verify", "E-Mail-Verifizierung für AS{asn}"),
    ("auth.authenticated", "Authentifiziert als AS{asn} ({mnt})"),
    ("auth.redirecting", "Weiterleitung\u{2026}"),
    ("auth.loading", "Laden\u{2026}"),
    (
        "auth.oidc_alt",
        "Oder melden Sie sich mit Ihrem Identitätsanbieter an und wir ermitteln Ihre ASN automatisch.",
    ),
    ("auth.continue_with", "Weiter mit {provider}"),
    (
        "auth.email_intro",
        "Senden Sie einen Anmeldelink und Einmalcode an die E-Mail-Kontakte eines Ihrer Maintainer, klicken Sie dann auf den Link oder geben Sie den Code ein.",
    ),
    ("auth.email_auth_as", "Authentifizierung als {mnt}"),
    ("auth.email_send_to", "Senden an {emails}"),
    (
        "auth.email_code_prompt",
        "Geben Sie den Auth-Code aus Ihrer E-Mail ein",
    ),
    (
        "auth.code_sent",
        "Wir haben einen Anmeldelink und Auth-Code an {emails} gesendet.",
    ),
    ("prompt.emails", "emails"),
    ("action.find_methods", "Authentifizierungsmethoden suchen"),
    ("action.verify", "Verifizieren"),
    ("action.verify_code", "Code verifizieren"),
    ("action.send_code", "Anmeldelink senden"),
    ("action.resend_code", "Anmeldelink erneut senden"),
    ("action.back", "Zurück"),
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
        "Führen Sie den obigen Befehl aus und fügen Sie dann Ihre SSH-Signatur ein.",
    ),
    (
        "pgp.paste_signed",
        "Fügen Sie Ihre vollständige clearsigned Nachricht ein.",
    ),
    (
        "pgp.paste_pubkey",
        "Fügen Sie Ihren ASCII-geschützten öffentlichen Schlüssel ein.",
    ),
    ("nav.language", "Sprache"),
    (
        "auth.oidc_redirecting",
        "Weiterleitung zu Ihrem OIDC-Anbieter\u{2026}",
    ),
    ("error.auth.return_to.invalid", "Ungültige Zieladresse."),
    ("auth_method.registry_email.label", "Registrierungs-E-Mail"),
    (
        "auth_method.host_impersonation.label",
        "Host-ASN-Nachahmung",
    ),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
