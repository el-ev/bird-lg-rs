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
    (
        "auth.complete_close",
        "Sie können diese Seite jetzt schließen.",
    ),
    ("auth.loading", "Laden\u{2026}"),
    (
        "auth.finding_methods",
        "Suche Authentifizierungsmethoden für Ihre ASN\u{2026}",
    ),
    (
        "auth.oidc_alt",
        "Oder melden Sie sich mit Ihrem Identitätsanbieter an und wir ermitteln Ihre ASN automatisch.",
    ),
    ("auth.continue_with", "Weiter mit {provider}"),
    (
        "auth.email_intro",
        "Wählen Sie einen Maintainer und wir senden einen Anmeldelink und Einmalcode an dessen Registry-E-Mail-Kontakte. Öffnen Sie dann den Link aus der E-Mail oder geben Sie den Code unten ein.",
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
    ("block.challenge", "Challenge"),
    ("block.sign_command", "Signaturbefehl"),
    (
        "block.export_command",
        "Befehl zum Exportieren des öffentlichen Schlüssels",
    ),
    ("nav.language", "Sprache"),
    (
        "auth.oidc_redirecting",
        "Weiterleitung zu Ihrem OIDC-Anbieter\u{2026}",
    ),
    ("error.auth.return_to.invalid", "Ungültige Zieladresse."),
    (
        "error.registry.lookup_failed",
        "Die Abfrage von AS{asn} in der dn42-Registry ist fehlgeschlagen. Bitte versuchen Sie es später erneut.",
    ),
    (
        "error.registry.unavailable",
        "Die dn42-Registry konnte nicht gelesen werden (Grund: {reason}). Das Problem liegt beim Auth-Dienst, nicht bei AS{asn}. Bitte versuchen Sie es später erneut oder kontaktieren Sie den Betreiber.",
    ),
    ("auth_method.registry_email.label", "E-Mail"),
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
