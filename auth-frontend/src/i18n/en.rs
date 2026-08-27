pub(super) const TABLE: &[(&str, &str)] = &[
    ("app.title", "dn42 Auth"),
    ("app.subtitle", "of IRIS-AS 4242421023"),
    (
        "auth.prompt",
        "Enter your dn42 ASN for registry SSH, PGP, or email auth.",
    ),
    (
        "auth.asn_methods",
        "We found registry auth methods for AS{asn}",
    ),
    ("auth.email_verify", "Email verification for AS{asn}"),
    ("auth.authenticated", "Authenticated as AS{asn} ({mnt})"),
    ("auth.redirecting", "Redirecting\u{2026}"),
    ("auth.complete_close", "You can close this page now."),
    ("auth.loading", "Loading\u{2026}"),
    (
        "auth.finding_methods",
        "Looking up auth methods for your ASN\u{2026}",
    ),
    (
        "auth.oidc_alt",
        "Or sign in with your identity provider and we will look up your ASN automatically.",
    ),
    ("auth.continue_with", "Continue with {provider}"),
    (
        "auth.email_intro",
        "Choose a maintainer and we will send a sign-in link and one-time code to its registry email contacts. Then open the link from the email, or paste the code below.",
    ),
    ("auth.email_auth_as", "Authenticate as {mnt}"),
    ("auth.email_send_to", "Send to {emails}"),
    (
        "auth.email_code_prompt",
        "Paste the auth code from your email",
    ),
    (
        "auth.code_sent",
        "We sent a sign-in link and auth code to {emails}.",
    ),
    ("prompt.emails", "emails"),
    ("action.find_methods", "Find Registry Auth Methods"),
    ("action.verify", "Verify"),
    ("action.verify_code", "Verify Code"),
    ("action.send_code", "Send Sign-In Link"),
    ("action.resend_code", "Resend Sign-In Link"),
    ("action.back", "Back"),
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
        "Run the command above, then paste your detached SSH signature.",
    ),
    (
        "pgp.paste_signed",
        "Paste your full clear-signed challenge.",
    ),
    ("pgp.paste_pubkey", "Paste your ASCII-armored public key."),
    ("block.challenge", "challenge"),
    ("block.sign_command", "signing command"),
    ("block.export_command", "public key export command"),
    ("nav.language", "Language"),
    (
        "auth.oidc_redirecting",
        "Redirecting you to your OIDC provider\u{2026}",
    ),
    ("error.auth.asn.required", "ASN is required."),
    (
        "error.auth.oidc.no_provider",
        "OIDC provider not configured.",
    ),
    (
        "error.auth.host_impersonation.unsupported",
        "Host impersonation is not supported in this flow.",
    ),
    (
        "error.auth.ssh.paste_signed_not_challenge",
        "Paste the signed signature, not the raw challenge text.",
    ),
    (
        "error.auth.ssh.paste_complete_block",
        "Paste the complete SSH signature block.",
    ),
    ("error.auth.pgp.paste_signed", "Paste the signed message."),
    ("error.auth.email.no_targets", "No email targets available."),
    (
        "error.auth.email.code_empty",
        "Enter the verification code.",
    ),
    (
        "error.auth.session.invalid",
        "Invalid authentication session.",
    ),
    ("error.auth.return_to.invalid", "Invalid return URL."),
    (
        "error.auth.asn.unsupported",
        "We currently support only ASNs in the 424242xxxx range.",
    ),
    (
        "error.auth.asn.not_found",
        "That ASN does not exist in the dn42 registry.",
    ),
    (
        "error.auth.asn.no_supported_auth",
        "That ASN exists in dn42, but it does not publish maintainer auth we can use yet.",
    ),
    (
        "error.auth.asn.no_registry_auth.oidc_hint",
        "That ASN does not expose supported registry SSH, PGP, or email auth methods. Use one of the configured OIDC login options instead.",
    ),
    (
        "error.auth.ssh.malformed_signature",
        "SSH signature data is malformed. Re-run ssh-keygen -Y sign and paste the full detached signature block.",
    ),
    (
        "error.auth.ssh.unrecognized_key",
        "Your SSH signature was made with a key that is not listed in any maintainer (mntner) object for this ASN.",
    ),
    (
        "error.auth.ssh.verification_failed",
        "SSH signature verification failed.",
    ),
    (
        "error.auth.pgp.verification_failed",
        "PGP signature verification failed. Re-sign the challenge with the matching registry key and paste the full signed block.",
    ),
    (
        "error.auth.pgp.unrecognized_key",
        "Your PGP fingerprint {fingerprint} is not listed in any maintainer (mntner) object for this ASN.",
    ),
    (
        "error.auth.challenge.expired",
        "Your authentication challenge has expired.",
    ),
    (
        "error.auth.registry_email.code.invalid",
        "Registry email auth code is invalid.",
    ),
    (
        "error.registry.lookup_failed",
        "Looking up AS{asn} in the dn42 registry failed. Please try again later.",
    ),
    (
        "error.registry.unavailable",
        "We could not read the dn42 registry (reason: {reason}). This is a problem on the auth service side, not with AS{asn}. Please try again later or contact the operator.",
    ),
    // Backend auth method copy (same keys the worker sends in UiMessage)
    ("auth_method.registry_ssh.label", "SSH Signature"),
    (
        "auth_method.registry_ssh.description",
        "Sign our challenge with an SSH key from your dn42 maintainer object.",
    ),
    ("auth_method.registry_pgp.label", "PGP Signature"),
    (
        "auth_method.registry_pgp.description",
        "Use one of your PGP fingerprints.",
    ),
    (
        "auth_method.registry_pgp.description_single",
        "Use your PGP fingerprint: {fingerprint}",
    ),
    ("auth_method.registry_email.label", "Email"),
    (
        "auth_method.registry_email.description",
        "Choose a maintainer and send a sign-in link to its registry email contacts.",
    ),
    (
        "auth_method.registry_email.description_single",
        "Send a sign-in link and one-time code to {emails}.",
    ),
    (
        "auth_method.host_impersonation.label",
        "Host ASN Impersonation",
    ),
    (
        "auth_method.host_impersonation.description",
        "You are impersonating {mnt} through our host ASN AS{host_asn}.",
    ),
    (
        "auth_method.oidc.description",
        "Authenticate with {provider} and prove one of your maintainer claims for this ASN.",
    ),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
