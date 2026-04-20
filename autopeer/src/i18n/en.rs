pub(super) const TABLE: &[(&str, &str)] = &[
    // App chrome
    ("app.title", "DN42 Autopeer"),
    ("app.title.footnote", "of IRIS-AS 4242421023"),
    ("nav.looking_glass", "Looking Glass"),
    ("nav.language", "Language"),
    // Generic actions
    ("action.back", "Back"),
    ("action.refresh", "Refresh"),
    ("action.logout", "Logout"),
    ("action.cancel_edit", "Cancel Edit"),
    ("action.choose_another_node", "Choose Another Node"),
    ("action.back_to_nodes", "Back To Nodes"),
    ("action.back_to_details", "Back To Details"),
    ("action.review_your_update", "Review Your Update"),
    ("action.review_your_change", "Review Your Change"),
    ("action.open_update_pr", "Open Update PR"),
    ("action.open_create_pr", "Open Create PR"),
    ("action.impersonate_this_asn", "Impersonate This ASN"),
    ("action.return_to_host_asn", "Return To Host ASN"),
    ("action.find_registry_auth", "Find Registry Auth Methods"),
    ("action.verify", "Verify"),
    ("action.verify_code", "Verify Code"),
    ("action.send_signin_link", "Send Sign-In Link"),
    ("action.resend_signin_link", "Resend Sign-In Link"),
    ("action.confirm_retirement", "Confirm Retirement"),
    ("action.retire_session", "Retire This Session"),
    ("action.open_pr", "Open PR"),
    ("action.workflow_run", "Workflow Run"),
    // Step: LoadingConfig / EnterAsn
    (
        "step.loading_config.prompt",
        "Loading runtime configuration",
    ),
    (
        "step.loading_config.message",
        "Loading runtime configuration...",
    ),
    (
        "step.enter_asn.prompt",
        "Enter your DN42 ASN for registry SSH, PGP, or email auth.",
    ),
    ("step.enter_asn.placeholder", "424242xxxx"),
    (
        "step.enter_asn.oidc_alt",
        "Or sign in with your identity provider and let us derive your ASN automatically.",
    ),
    ("step.enter_asn.continue_with", "Continue with {provider}"),
    // Step: SelectMethod
    (
        "step.select_method.found_for_as",
        "We found registry auth methods for AS{asn}",
    ),
    // Backend auth method copy
    ("auth_method.registry_ssh.label", "Registry SSH Signature"),
    (
        "auth_method.registry_ssh.description",
        "Sign our challenge with an SSH key from your DN42 maintainer object.",
    ),
    ("auth_method.registry_pgp.label", "Registry PGP Signature"),
    (
        "auth_method.registry_pgp.description",
        "Use one of your registry PGP fingerprints: {fingerprints}",
    ),
    ("auth_method.registry_email.label", "Registry Email"),
    (
        "auth_method.registry_email.description",
        "Choose a maintainer and send a sign-in link to its registry email contacts.",
    ),
    (
        "auth_method.registry_email.description_single",
        "Send a sign-in link and one-time code to {emails}.",
    ),
    (
        "auth_method.registry_ssh.session_description",
        "You authenticated with {mnt} using registry SSH auth.",
    ),
    (
        "auth_method.registry_pgp.session_description",
        "You authenticated with {mnt} using registry PGP auth.",
    ),
    (
        "auth_method.registry_email.session_description",
        "You authenticated with {mnt} using registry email auth.",
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
    (
        "auth_method.oidc.session_description",
        "You authenticated with {provider} as {mnt}.",
    ),
    // Step: VerifyMethod (SSH)
    (
        "verify.ssh.no_fingerprints",
        "We could not find any registry SSH key fingerprints for your ASN.",
    ),
    ("verify.ssh.match_one", "Match your SSH key {fingerprint}"),
    (
        "verify.ssh.match_many",
        "Match one of your SSH keys: {fingerprints}",
    ),
    ("verify.ssh.create_signature", "Sign the challenge"),
    (
        "verify.ssh.paste_prompt",
        "Run the command above, then paste your detached SSH signature",
    ),
    ("verify.ssh.placeholder", "-----BEGIN SSH SIGNATURE-----"),
    // Step: VerifyMethod (PGP)
    (
        "verify.pgp.no_fingerprints",
        "We could not find any registry PGP fingerprints for your ASN.",
    ),
    ("verify.pgp.use_key", "Use your key {fingerprint}"),
    (
        "verify.pgp.clearsign_intro",
        "Clear-sign the exact challenge text with your matching key, then export that same public key and paste both outputs below.",
    ),
    ("verify.pgp.exact_challenge", "Exact challenge text"),
    ("verify.pgp.clearsign_label", "Clear-sign your challenge"),
    (
        "verify.pgp.signed_paste_prompt",
        "Paste your full clear-signed challenge from the command above",
    ),
    (
        "verify.pgp.signed_placeholder",
        "-----BEGIN PGP SIGNED MESSAGE-----",
    ),
    ("verify.pgp.export_label", "Export your public key"),
    (
        "verify.pgp.pubkey_paste_prompt",
        "Paste your ASCII-armored public key from the export command above",
    ),
    (
        "verify.pgp.pubkey_placeholder",
        "-----BEGIN PGP PUBLIC KEY BLOCK-----",
    ),
    // Step: VerifyMethod (Email)
    (
        "verify.email.intro",
        "Send a sign-in link and one-time code to the registry email contacts on one of your maintainers, then click the link or paste the code below.",
    ),
    (
        "verify.email.no_contacts",
        "We could not find any admin-c or tech-c email contacts for your ASN.",
    ),
    ("verify.email.auth_as", "Authenticate as {mnt}"),
    ("verify.email.send_to", "Send to {emails}"),
    (
        "verify.email.sent_to_prefix",
        "We sent a sign-in link and auth code to {emails}.",
    ),
    (
        "verify.email.code_prompt",
        "Paste the auth code from your email",
    ),
    ("verify.email.code_placeholder", "12345678"),
    // Step: VerifyMethod (OIDC / Host)
    ("verify.oidc.continue_to", "Continue to {provider}"),
    (
        "verify.oidc.in_browser",
        "Continue to {provider} in your browser",
    ),
    (
        "verify.oidc.redirect_note",
        "We will redirect you to your provider, then bring you back here after it proves your ASN and maintainer claims.",
    ),
    (
        "verify.host.note",
        "Impersonation is available after you authenticate one of our configured host ASNs.",
    ),
    (
        "verify.choose_first",
        "Choose an authentication method first.",
    ),
    ("verify.auth_for_as", "{label} for AS{asn}"),
    // Manage / dashboard headings
    ("dashboard.flow_kicker", "Your Peering Flow"),
    (
        "dashboard.host_readonly_title",
        "Our host ASN stays read-only here",
    ),
    ("dashboard.update_managed_title", "Update your session"),
    (
        "dashboard.create_or_manage_title",
        "Create or manage your sessions",
    ),
    (
        "dashboard.host_readonly_body",
        "Our host ASN is only for supporting other networks. Before you create, update, or retire sessions, impersonate the ASN you want to manage.",
    ),
    (
        "dashboard.create_or_manage_body",
        "Authenticate once and choose one of our nodes. From there you can create a new session, or open an existing one to update or retire it.",
    ),
    ("dashboard.session_badge_template", "{mnt} via {label}"),
    // Sidebar
    ("sidebar.your_session_kicker", "Your Session"),
    ("sidebar.no_active_session", "No active session"),
    (
        "sidebar.session_authed_template",
        "You authenticated as {mnt} via {label}.",
    ),
    ("sidebar.support_kicker", "Support Mode"),
    ("sidebar.host_asn_prefix", "Host ASN AS{asn}"),
    (
        "sidebar.host_authed_template",
        "You authenticated as {mnt} via {label}. Use this only when you need to open or repair sessions for another ASN.",
    ),
    ("sidebar.impersonate_asn_label", "impersonate_asn"),
    ("sidebar.effective_mnt_label", "effective_mnt"),
    ("sidebar.impersonate_asn_placeholder", "424242xxxx"),
    (
        "sidebar.impersonate_mnt_placeholder",
        "Optional mntner override",
    ),
    ("sidebar.current_operation", "Current Operation"),
    ("sidebar.support_mode_title", "Impersonate another ASN"),
    (
        "sidebar.support_mode_body",
        "This host ASN is only for helping other networks. Use the controls on the right to impersonate the ASN you want to manage.",
    ),
    // Stage 1: Select node
    ("stage1.kicker", "Stage 1"),
    ("stage1.title", "Choose one of our nodes"),
    ("flow.select_node.title", "Choose Node"),
    (
        "flow.select_node.description",
        "Choose the nearest node in our network before you fill in tunnel details.",
    ),
    ("flow.session_details.title", "Configure Your Session"),
    (
        "flow.session_details.description",
        "Enter your WireGuard and BGP values, then adjust any advanced options you need.",
    ),
    ("flow.review.title", "Review Your Change"),
    (
        "flow.review.description",
        "Review your change before we open the pull request.",
    ),
    (
        "stage1.description",
        "Choose a node in our network. Empty nodes let you create a session; existing sessions open in place for updates. Manual sessions get adopted into autopeer automatically when you save. In-flight nodes stay read-only.",
    ),
    (
        "stage1.empty_title",
        "We did not find any autopeer-enabled nodes for your ASN.",
    ),
    (
        "stage1.empty_body",
        "If that looks wrong, refresh the page or check our autopeer policy.",
    ),
    ("stage1.state.available", "Available"),
    ("stage1.state.disabled", "Disabled"),
    (
        "stage1.state.note.create",
        "Create your session on this node.",
    ),
    (
        "stage1.state.note.managed",
        "Open this node to update or retire your session.",
    ),
    (
        "stage1.state.note.manual",
        "Open this node to review the current repo config. Saving it will adopt the session into autopeer automatically.",
    ),
    (
        "stage1.state.note.pending",
        "A change for your session is already in progress here.",
    ),
    (
        "stage1.state.note.conflict",
        "Our repo is in conflict for this node.",
    ),
    (
        "stage1.state.note.disabled",
        "This node is not accepting autopeer sessions right now.",
    ),
    // Stage 2: Session details
    ("stage2.kicker", "Stage 2"),
    (
        "stage2.title.update_prefix",
        "Update or retire your session on {node}",
    ),
    (
        "stage2.title.create_prefix",
        "Set up your session on {node}",
    ),
    ("stage2.title.create_blank", "Set up your new session"),
    (
        "stage2.update_intro",
        "You already have a managed session on this node. Update your peering details below, or retire the session if you no longer want it here.",
    ),
    ("stage2.section.connection", "Connection"),
    ("stage2.section.tunnel", "Tunnel Addresses"),
    (
        "stage2.section.tunnel.help",
        "Use the addresses you configured on your side. IPv6 can be either ULA like `fd55:...` or link-local like `fe80:...`.",
    ),
    ("stage2.section.families", "Route Families"),
    (
        "stage2.section.families.help",
        "Choose which DN42 route families your session should carry.",
    ),
    ("stage2.section.bgp", "BGP Behavior"),
    (
        "stage2.section.bgp.help",
        "MP-BGP uses your IPv6 address for a single BGP session that can carry IPv4 and/or IPv6 routes. If you disable it, we will generate separate BGP sessions. Extended Next Hop only applies to MP-BGP.",
    ),
    ("stage2.section.policy", "Routing Policy"),
    ("stage2.advanced.summary", "Advanced options"),
    ("stage2.field.endpoint", "Endpoint"),
    (
        "stage2.field.endpoint.placeholder",
        "Hostname or IP:port of your router",
    ),
    ("stage2.field.wg_key", "WireGuard key"),
    (
        "stage2.field.wg_key.placeholder",
        "Base64 public key from your router",
    ),
    ("stage2.field.peer4", "Peer IPv4 address"),
    (
        "stage2.field.peer4.placeholder",
        "Optional DN42 IPv4 address on your side",
    ),
    ("stage2.field.peer6", "Peer IPv6 address"),
    (
        "stage2.field.peer6.placeholder",
        "ULA or link-local, e.g. fd55:dead:beef::3 or fe80::1234",
    ),
    ("stage2.field.own6_link_local", "Our link-local IPv6"),
    (
        "stage2.field.own6_link_local.placeholder",
        "Only needed when your peer IPv6 address is link-local",
    ),
    ("stage2.field.own6_node", "Our node IPv6"),
    (
        "stage2.field.own6_node.no_inventory",
        "Our inventory doesn't list an IPv6 address for this node.",
    ),
    ("stage2.field.families", "Families"),
    ("stage2.field.families.ipv4_label", "IPv4 routes"),
    ("stage2.field.families.ipv6_label", "IPv6 routes"),
    ("stage2.field.bgp_features", "Features"),
    ("stage2.field.bgp.mpbgp_label", "MP-BGP"),
    ("stage2.field.bgp.enh_label", "Extended Next Hop"),
    ("stage2.field.policy", "Policy"),
    ("stage2.field.comment", "Comment"),
    (
        "stage2.field.comment.placeholder",
        "Optional note about your session",
    ),
    ("stage2.field.keepalive", "Persistent keepalive"),
    (
        "stage2.field.keepalive.placeholder",
        "Optional keepalive in seconds for your router",
    ),
    ("stage2.field.mtu", "Interface MTU"),
    ("stage2.field.mtu.placeholder", "Optional MTU"),
    // Stage 3: Review
    ("stage3.kicker", "Stage 3"),
    ("stage3.title", "Review your change before we open the PR"),
    ("stage3.review.our_node", "Our node"),
    ("stage3.review.not_selected", "Not selected"),
    ("stage3.review.endpoint", "Endpoint"),
    ("stage3.review.wg_key", "WireGuard Public key"),
    ("stage3.review.route_families", "Route families"),
    ("stage3.review.bgp_behavior", "BGP behavior"),
    ("stage3.review.bgp.mpbgp", "MP-BGP"),
    ("stage3.review.bgp.separate", "Separate IPv4/IPv6 sessions"),
    ("stage3.review.bgp.enh_suffix", " + Extended Next Hop"),
    ("stage3.review.routing_policy", "Routing policy"),
    ("stage3.review.peer4", "Peer IPv4 address"),
    ("stage3.review.peer6", "Peer IPv6 address"),
    ("stage3.review.own6", "Our link-local IPv6"),
    ("stage3.review.keepalive", "Persistent keepalive"),
    ("stage3.review.mtu", "MTU"),
    ("stage3.review.note", "Your note"),
    ("stage3.review.our_node_details", "Our node details"),
    ("stage3.review.our_endpoint", "Endpoint"),
    ("stage3.review.our_ipv4", "IPv4"),
    ("stage3.review.our_ipv6", "IPv6"),
    ("stage3.review.our_link_local_ipv6", "Link-local IPv6"),
    ("stage3.review.our_wg_pubkey", "WireGuard public key"),
    ("stage3.review.our_node_note", "Note"),
    // Draft / node formatting
    ("draft.families.ipv4_ipv6", "IPv4 + IPv6"),
    ("draft.families.ipv4_only", "IPv4 only"),
    ("draft.families.ipv6_only", "IPv6 only"),
    ("draft.families.none", "No families selected"),
    ("location.direction.n", "North"),
    ("location.direction.s", "South"),
    ("location.direction.e", "East"),
    ("location.direction.w", "West"),
    ("location.direction.ne", "Northeast"),
    ("location.direction.nw", "Northwest"),
    ("location.direction.se", "Southeast"),
    ("location.direction.sw", "Southwest"),
    ("node.transport.ipv4", "IPv4 transport"),
    ("node.transport.ipv6", "IPv6 transport"),
    ("node.transport.dual_stack", "Dual-stack transport"),
    // Session / operation labels
    ("session_state.managed", "Managed"),
    ("session_state.manual", "Manual"),
    ("session_state.pending_pr", "Pending PR"),
    ("session_state.conflict", "Conflict"),
    ("operation.kind.create", "Create"),
    ("operation.kind.update", "Update"),
    ("operation.kind.delete", "Delete"),
    ("operation.kind.migrate", "Migrate"),
    ("operation.state.pending_pull_request", "Preparing PR"),
    ("operation.state.pending_checks", "Waiting For CI"),
    ("operation.state.applying", "Applying On Node"),
    ("operation.state.pending_merge", "Waiting For Merge"),
    ("operation.state.completed", "Completed"),
    ("operation.state.failed", "Failed"),
    ("operation.state.conflict", "Conflict"),
    // Backend operation messages
    (
        "operation.message.pending_pull_request",
        "We are preparing your pull request.",
    ),
    (
        "operation.message.pending_checks",
        "Your pull request is open; waiting for peer-session-check.",
    ),
    (
        "operation.message.applying",
        "Checks passed; applying your session to the node for verification.",
    ),
    (
        "operation.message.pending_merge",
        "Apply succeeded on the node; waiting for merge.",
    ),
    (
        "operation.message.completed",
        "Your change was applied and merged successfully.",
    ),
    ("operation.message.failed", "Your change failed."),
    (
        "operation.message.conflict",
        "We could not apply your change because our repo conflicted.",
    ),
    (
        "operation.message.wait_node_lock",
        "Apply succeeded; waiting for another change on this node to finish merging.",
    ),
    (
        "operation.message.no_change",
        "Your session already matches our repo, so we did not open a pull request.",
    ),
    (
        "operation.message.check_not_started",
        "peer-session-check did not start for your pull request.",
    ),
    (
        "operation.message.check_wait_start",
        "Your pull request is open; waiting for peer-session-check to start.",
    ),
    (
        "operation.message.check_failed",
        "peer-session-check finished with {conclusion}.",
    ),
    (
        "operation.message.apply_not_started",
        "peer-session-apply did not start for your pull request.",
    ),
    (
        "operation.message.apply_wait_start",
        "Checks passed; waiting for peer-session-apply to start.",
    ),
    (
        "operation.message.apply_failed",
        "peer-session-apply finished with {conclusion}.",
    ),
    (
        "operation.message.pull_request_closed",
        "Your pull request was closed before merge.",
    ),
    (
        "operation.message.merge_failed",
        "Waiting for merge. Merge attempt failed: {error}",
    ),
    ("operation.failure_stage.checks", "CI checks"),
    ("operation.failure_stage.preflight", "Node preflight"),
    ("operation.failure_stage.apply", "Node apply"),
    ("operation.failure_stage.merge", "Merge"),
    // Routing policy labels
    ("peering_strategy.full_table.label", "Full Table"),
    (
        "peering_strategy.full_table.description",
        "Receive all valid routes and export all valid routes.",
    ),
    ("peering_strategy.transit.label", "Transit"),
    (
        "peering_strategy.transit.description",
        "Receive all valid routes and export only our own exact prefixes.",
    ),
    ("peering_strategy.peer.label", "Peer"),
    (
        "peering_strategy.peer.description",
        "Receive only direct routes and export our own exact prefixes plus downstream routes.",
    ),
    ("peering_strategy.downstream.label", "Downstream"),
    (
        "peering_strategy.downstream.description",
        "Receive only direct routes and export all valid routes.",
    ),
    // Operation progress labels
    ("operation.progress.branch", "Branch"),
    ("operation.progress.checks", "Checks"),
    ("operation.progress.apply", "Apply"),
    ("operation.progress.merge", "Merge"),
    ("operation.progress.done", "Done"),
    // Operation failure labels
    ("operation.failure.stage", "Failed stage"),
    ("operation.failure.conclusion", "Result"),
    // Prompts (shell-style left labels)
    ("prompt.autopeer", "autopeer"),
    ("prompt.asn", "ASN"),
    ("prompt.auth", "auth"),
    ("prompt.login", "login"),
    ("prompt.key", "key"),
    ("prompt.keys", "keys"),
    ("prompt.signature", "signature"),
    ("prompt.signed", "signed"),
    ("prompt.pubkey", "pubkey"),
    ("prompt.mntner", "mntner"),
    ("prompt.emails", "emails"),
    ("prompt.code", "code"),
    // Generic loading / errors
    ("status.working", "Working..."),
    (
        "error.ui.node.choose",
        "Choose one of our nodes before you continue",
    ),
    (
        "error.ui.node.choose_inline",
        "Choose one of our nodes before you open a PR",
    ),
    (
        "error.ui.session.missing_config",
        "Your current session is missing config details",
    ),
    (
        "error.ui.operation.wait_inflight",
        "An in-flight change is still running on this node — wait for it to finish.",
    ),
    (
        "error.ui.node.blocked_conflict",
        "This node is blocked by a conflict in our repo",
    ),
    (
        "error.ui.session.choose_managed_to_retire",
        "Choose one of your sessions before you retire it",
    ),
    (
        "error.ui.auth.authenticate_first",
        "Authenticate before you continue",
    ),
    (
        "error.ui.node.choose_default",
        "Choose one of our nodes before you continue",
    ),
    ("error.auth.asn.required", "ASN is required"),
    (
        "error.auth.oidc.provider.missing",
        "OIDC provider is missing",
    ),
    ("error.request.challenge_id.missing", "Missing challenge_id"),
    (
        "error.ui.auth.method.choose_first",
        "Choose an authentication method first.",
    ),
    (
        "error.ui.auth.registry_email.inactive",
        "Registry email auth is not active right now.",
    ),
    (
        "error.ui.auth.registry_email.choose_maintainer",
        "Choose a maintainer with registry email contacts first.",
    ),
    (
        "error.ui.auth.registry_email.code.required",
        "Enter the one-time auth code from your email.",
    ),
    (
        "error.auth.method.unavailable",
        "That authentication method is no longer available for your ASN.",
    ),
    (
        "error.ui.auth.impersonation.host_auth_first",
        "Authenticate one of our configured host ASNs before you impersonate another ASN.",
    ),
    (
        "error.ui.auth.impersonation.asn.required",
        "Enter the ASN you want to impersonate",
    ),
    (
        "error.ui.auth.impersonation.host_session.missing",
        "No host ASN session is available right now",
    ),
    (
        "error.ui.auth.impersonation.host_required",
        "Impersonation is only available after you authenticate one of our configured host ASNs.",
    ),
    (
        "error.runtime.browser.unavailable",
        "Browser window is unavailable",
    ),
    (
        "error.runtime.oidc.redirect_failed",
        "Failed to open the OIDC login redirect",
    ),
    (
        "error.runtime.config.autopeer_url.missing",
        "autopeer_url is not configured",
    ),
    (
        "error.auth.ssh.empty_or_missing_blocks",
        "Paste the full detached SSH signature block from the command above, including the BEGIN/END lines.",
    ),
    (
        "error.auth.ssh.unsigned_challenge",
        "Paste the detached SSH signature block from the command above, not the unsigned challenge text.",
    ),
    (
        "error.request.body.invalid_json",
        "Request body must be valid JSON.",
    ),
    (
        "error.auth.asn.unsupported",
        "We do not support that ASN range yet. Right now Autopeer only supports 424242xxxx.",
    ),
    (
        "error.auth.asn.not_found",
        "AS{asn} is invalid because it does not exist in the DN42 registry.",
    ),
    (
        "error.auth.asn.no_supported_auth",
        "AS{asn} exists in DN42, but it does not publish maintainer auth we can use yet.",
    ),
    (
        "error.auth.registry_email.unavailable",
        "Registry email login is not available in this deployment.",
    ),
    ("error.auth.challenge.unknown_id", "Unknown challenge_id."),
    (
        "error.auth.challenge.expired",
        "Your authentication challenge has expired.",
    ),
    (
        "error.auth.challenge.used",
        "This authentication challenge has already been used.",
    ),
    (
        "error.auth.session.token.missing",
        "Missing bearer session token.",
    ),
    ("error.auth.session.unknown", "Unknown auth session."),
    ("error.auth.session.expired", "Auth session has expired."),
    (
        "error.auth.impersonation.no_maintainers",
        "This ASN has no maintainers available for impersonation.",
    ),
    (
        "error.auth.ssh.malformed_signature",
        "SSH signature data is malformed. Re-run ssh-keygen -Y sign and paste the full detached signature block.",
    ),
    (
        "error.auth.ssh.unrecognized_key",
        "Your SSH signature used a key that is not present in the resolved maintainer objects.",
    ),
    (
        "error.auth.ssh.verification_failed",
        "SSH signature verification failed.",
    ),
    (
        "error.auth.pgp.invalid_public_key",
        "PGP public key is invalid. Export your ASCII-armored public key and paste the full block.",
    ),
    (
        "error.auth.pgp.invalid_signed_message",
        "PGP signed message is invalid. Clear-sign the challenge and paste the full signed block.",
    ),
    (
        "error.auth.pgp.verification_failed",
        "PGP signature verification failed. Re-sign the challenge with the matching registry key and paste the full signed block.",
    ),
    (
        "error.auth.pgp.unrecognized_key",
        "Your PGP fingerprint {fingerprint} is not present in the resolved maintainer objects.",
    ),
    (
        "error.auth.pgp.challenge_mismatch",
        "Your PGP signed message does not match the issued challenge.",
    ),
    (
        "error.auth.registry_email.state.missing",
        "Registry email login state was not found or has expired.",
    ),
    (
        "error.auth.registry_email.state.expired",
        "Registry email login has expired.",
    ),
    (
        "error.auth.registry_email.state.pending",
        "Registry email login has not completed yet.",
    ),
    (
        "error.auth.registry_email.code.invalid",
        "Registry email auth code is invalid.",
    ),
    (
        "error.auth.registry_email.session.missing",
        "Registry email login session is no longer available.",
    ),
    (
        "error.auth.registry_email.session.expired",
        "Registry email login session has expired.",
    ),
    (
        "error.auth.registry_email.callback.params.missing",
        "Missing registry email callback parameters.",
    ),
    (
        "error.auth.registry_email.callback.failed",
        "Registry email login failed; please try again.",
    ),
    (
        "error.auth.registry_email.contacts.missing",
        "AS{asn} does not expose any admin-c or tech-c email addresses we can use in the registry.",
    ),
    (
        "error.auth.registry_email.target.missing",
        "{requested} does not have registry email contacts we can use for this ASN.",
    ),
    (
        "error.auth.registry_email.target.required",
        "effective_mnt is required when your registry email auth covers multiple maintainers.",
    ),
    (
        "error.auth.oidc.callback.provider.missing",
        "OIDC provider is missing from the callback path.",
    ),
    (
        "error.auth.oidc.callback.params.missing",
        "Missing OIDC callback parameters.",
    ),
    (
        "error.auth.oidc.provider.unknown",
        "Unknown OIDC provider {provider}.",
    ),
    (
        "error.auth.oidc.provider.rejected",
        "{error}: {description}",
    ),
    (
        "error.auth.oidc.state.missing",
        "OIDC login state was not found or has expired.",
    ),
    (
        "error.auth.oidc.state.expired",
        "OIDC login state has expired.",
    ),
    (
        "error.auth.oidc.state.pending",
        "OIDC login has not completed yet.",
    ),
    (
        "error.auth.oidc.session.missing",
        "OIDC login session is no longer available.",
    ),
    (
        "error.auth.oidc.session.expired",
        "OIDC login session has expired.",
    ),
    (
        "error.auth.oidc.callback.failed",
        "OIDC login failed; please try again.",
    ),
    (
        "error.auth.oidc.identity.asn_mismatch",
        "OIDC identity ASN {token_asn} does not match requested ASN {requested_asn}.",
    ),
    (
        "error.auth.session.path_asn_mismatch",
        "The path ASN does not match your authenticated session.",
    ),
    ("error.request.node.required", "Node is required."),
    (
        "error.request.session_payload.required",
        "Session payload is required.",
    ),
    (
        "error.auth.impersonation.maintainer.required",
        "effective_mnt is required when your target ASN has multiple maintainers. Available mntners: {available}.",
    ),
    (
        "error.auth.impersonation.maintainer.missing",
        "{requested} is not present in aut-num -> mnt-by for this ASN. Available mntners: {available}.",
    ),
    ("error.request.operation.not_found", "Operation not found."),
    ("error.request.route.not_found", "Not found."),
    // Frontend validation
    (
        "validation.tunnel.required",
        "Add at least one tunnel address: IPv4 or IPv6",
    ),
    (
        "validation.bgp_family.required",
        "Enable at least one BGP family",
    ),
    (
        "validation.peer4.required",
        "An IPv4 peer address is required for IPv4 when MP-BGP is disabled",
    ),
    (
        "validation.peer6.required_mp_bgp",
        "An IPv6 peer address is required when MP-BGP is enabled",
    ),
    (
        "validation.peer6.required_ipv6",
        "An IPv6 peer address is required for IPv6 routes",
    ),
    (
        "validation.extended_next_hop.requires_mp_bgp",
        "Extended next hop requires MP-BGP",
    ),
    (
        "validation.own6.requires_peer6",
        "A local link-local IPv6 needs a peer IPv6 address",
    ),
    (
        "validation.own6.requires_link_local_peer6",
        "Local link-local IPv6 only applies when the peer IPv6 address is link-local",
    ),
    (
        "validation.own6.must_start_fe80",
        "Local link-local IPv6 must start with fe80:",
    ),
    ("validation.endpoint.required", "endpoint is required"),
    (
        "validation.endpoint.no_spaces",
        "Remote endpoint cannot contain spaces",
    ),
    (
        "validation.endpoint.ipv6_format",
        "IPv6 endpoints must use the format [addr]:port",
    ),
    (
        "validation.endpoint.ipv6_invalid",
        "Remote endpoint IPv6 address must be a valid IPv6 address",
    ),
    (
        "validation.endpoint.host_port_format",
        "Remote endpoint must use host:port or [ipv6]:port",
    ),
    (
        "validation.endpoint.port_required",
        "Remote endpoint must include a port",
    ),
    (
        "validation.endpoint.host_required",
        "Remote endpoint host is required",
    ),
    (
        "validation.endpoint.host_invalid",
        "Remote endpoint host must be an IPv4 address or a fully qualified hostname",
    ),
    (
        "validation.endpoint.port.invalid",
        "Remote endpoint port must be a valid number",
    ),
    (
        "validation.endpoint.port.range",
        "Remote endpoint port must be between 1 and 65535",
    ),
    (
        "validation.wg_public_key.required",
        "wg_public_key is required",
    ),
    (
        "validation.wg_public_key.length",
        "Peer WireGuard key must be a 44-character base64 public key",
    ),
    (
        "validation.wg_public_key.suffix",
        "Peer WireGuard key must end with '='",
    ),
    (
        "validation.wg_public_key.charset",
        "Peer WireGuard key contains invalid base64 characters",
    ),
    (
        "validation.peer4.invalid",
        "Peer IPv4 address must be a valid IPv4 address",
    ),
    (
        "validation.peer4.range",
        "Peer IPv4 address must be within 172.20.0.0/14",
    ),
    (
        "validation.peer6.invalid",
        "Peer IPv6 address must be a valid IPv6 address",
    ),
    (
        "validation.peer6.scope",
        "Peer IPv6 address must be a ULA or link-local IPv6 address",
    ),
    (
        "validation.own6.invalid",
        "Local link-local IPv6 must be a valid IPv6 address",
    ),
    (
        "validation.own6.scope",
        "Local link-local IPv6 must be a link-local IPv6 address",
    ),
    (
        "validation.keepalive.invalid",
        "Persistent keepalive must be a valid number",
    ),
    (
        "validation.mtu.invalid",
        "Interface MTU must be a valid number",
    ),
    (
        "validation.mtu.range",
        "Interface MTU must be between 1280 and 1500",
    ),
    // Loading messages
    (
        "loading.email_login",
        "Finishing your email login and loading your sessions...",
    ),
    (
        "loading.oidc_login",
        "Finishing your OIDC login and loading your sessions...",
    ),
    (
        "loading.fetch_sessions",
        "Fetching your current sessions from our repo...",
    ),
    (
        "loading.refresh_sessions",
        "Refreshing your session state from our repo...",
    ),
    (
        "loading.fetch_methods",
        "Fetching your DN42 registry authentication methods...",
    ),
    (
        "loading.redirect_oidc",
        "Redirecting you to your OIDC provider...",
    ),
    (
        "loading.fetch_challenge",
        "Fetching a fresh DN42 registry challenge for you...",
    ),
    (
        "loading.send_email",
        "Sending a sign-in link and one-time code to your registry email contacts...",
    ),
    (
        "loading.check_ssh",
        "Checking your SSH signature against the DN42 registry...",
    ),
    (
        "loading.check_pgp",
        "Checking your PGP signature against the DN42 registry...",
    ),
    (
        "loading.check_email",
        "Checking your registry email auth code...",
    ),
    (
        "loading.host_session_prep",
        "Preparing your host ASN session...",
    ),
    (
        "loading.authing_asn",
        "Authenticating the ASN against the DN42 registry...",
    ),
    (
        "loading.restore_host",
        "Restoring your host ASN session from our repo...",
    ),
    (
        "loading.update_pr",
        "Updating your peering config in our repo and opening a pull request...",
    ),
    (
        "loading.create_pr",
        "Creating your peering config in our repo and opening a pull request...",
    ),
    (
        "loading.retire_pr",
        "Retiring your session in our repo and opening a pull request...",
    ),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
