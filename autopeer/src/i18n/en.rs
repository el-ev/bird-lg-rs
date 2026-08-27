pub(super) const TABLE: &[(&str, &str)] = &[
    // App chrome
    ("app.title", "dn42 Autopeer"),
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
    ("action.confirm_retirement", "Confirm Retirement"),
    ("action.retire_session", "Retire This Session"),
    ("action.confirm_deletion", "Confirm Deletion"),
    ("action.delete_session", "Delete This Session"),
    ("action.open_pr", "Open PR"),
    ("action.workflow_run", "Workflow Run"),
    ("action.retry", "Retry"),
    ("action.redeploy", "Re-deploy"),
    ("action.drop_changes", "Drop Changes"),
    ("action.dismiss_operation", "Dismiss"),
    ("action.check_bgp_session", "Check BGP Session"),
    // Step: LoadingConfig / AuthRedirect
    (
        "step.loading_config.prompt",
        "Loading runtime configuration\u{2026}",
    ),
    (
        "step.auth_redirect.prompt",
        "Authenticate and manage your peering sessions with IRIS-AS 4242421023.",
    ),
    ("step.auth_redirect.link", "Redirect to dn42-auth.owo.li"),
    // Auth method labels returned by auth service sessions
    ("auth_method.registry_ssh.label", "Registry SSH Signature"),
    ("auth_method.registry_pgp.label", "Registry PGP Signature"),
    ("auth_method.registry_email.label", "Registry Email"),
    (
        "auth_method.host_impersonation.label",
        "Host ASN Impersonation",
    ),
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
        "stage1.state.note.locked",
        "This session has been locked and cannot be modified through autopeer.",
    ),
    (
        "stage1.state.note.pending",
        "A change for your session is already in progress here.",
    ),
    (
        "stage1.state.note.stalled",
        "A previous deployment failed — open to modify, re-deploy, or drop.",
    ),
    (
        "stage1.state.note.conflict",
        "Our repo is in conflict for this node.",
    ),
    (
        "stage1.state.note.disabled",
        "This node is not accepting autopeer sessions right now.",
    ),
    // Stalled PR banner
    ("stalled.banner.title", "Deployment Failed"),
    (
        "stalled.banner.body",
        "A previous change has an open PR that failed to deploy. You can modify the config and submit again, re-deploy the existing PR, or drop the changes entirely.",
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
        "Use the addresses you configured on your side. IPv6 can be either ULA like `fd42:...` or link-local like `fe80:...`.",
    ),
    ("stage2.section.families", "Route Families"),
    (
        "stage2.section.families.help",
        "Choose which dn42 route families your session should carry.",
    ),
    ("stage2.section.bgp", "BGP Behavior"),
    (
        "stage2.section.bgp.help",
        "MP-BGP uses a single BGP session over your selected IPv4 or IPv6 transport to carry IPv4 and/or IPv6 routes; if you disable it, we will generate separate BGP sessions, and Extended Next Hop only applies to IPv4 routes carried over IPv6 transport.",
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
        "Your dn42 IPv4 address, e.g. 172.21.111.111",
    ),
    ("stage2.field.peer6", "Peer IPv6 address"),
    (
        "stage2.field.peer6.placeholder",
        "Your ULA or link-local, e.g. fd42:4242:1023:: or fe80::",
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
    ("stage2.field.own4_node", "Our node IPv4"),
    (
        "stage2.field.own4_node.no_inventory",
        "Our inventory doesn't list an IPv4 address for this node.",
    ),
    ("stage2.field.families", "Families"),
    ("stage2.field.families.ipv4_label", "IPv4 routes"),
    ("stage2.field.families.ipv6_label", "IPv6 routes"),
    ("stage2.field.bgp_features", "Features"),
    ("stage2.field.bgp.mpbgp_label", "MP-BGP"),
    ("stage2.field.bgp.enh_label", "Extended Next Hop"),
    ("stage2.field.bgp.transport", "Transport"),
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
    ("stage2.field.psk", "Pre-shared key"),
    ("stage2.field.psk.placeholder", "Optional WireGuard PSK"),
    (
        "stage2.field.psk.placeholder.existing",
        "PSK configured — leave empty to keep",
    ),
    ("stage2.field.psk.clear", "Clear PSK"),
    ("stage2.field.psk.generate", "Generate PSK"),
    ("stage2.field.psk.copied", "Copied"),
    (
        "stage2.field.psk.help",
        "An optional WireGuard pre-shared key for added security. The key will be encrypted before storage.",
    ),
    ("stage2.field.encrypt_endpoint", "Encrypted"),
    (
        "stage2.field.encrypt_endpoint.help",
        "Encrypt your endpoint address in the git repository so it is not visible in plaintext.",
    ),
    (
        "stage2.field.encrypt_endpoint.requires_endpoint",
        "takes effect once an endpoint is set",
    ),
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
    ("stage3.review.psk", "Pre-shared key"),
    ("stage3.review.psk.set", "Configured (encrypted)"),
    ("stage3.review.psk.not_set", "Not set"),
    ("stage3.review.psk.unchanged", "Configured (unchanged)"),
    ("stage3.review.psk.cleared", "Will be removed"),
    ("stage3.review.encrypt_endpoint.enabled", "Encrypted"),
    ("stage3.review.note", "Your note"),
    ("stage3.review.our_node_details", "Our node details"),
    ("stage3.review.our_endpoint", "Endpoint"),
    ("stage3.review.our_ipv4", "IPv4"),
    ("stage3.review.our_ipv6", "IPv6"),
    ("stage3.review.our_link_local_ipv6", "Link-local IPv6"),
    ("stage3.review.our_wg_pubkey", "WireGuard public key"),
    ("stage3.review.our_node_note", "Note"),
    ("stage3.review.check_session_label", "BGP session"),
    // Draft / node formatting
    ("draft.families.ipv4_ipv6", "IPv4 + IPv6"),
    ("draft.families.ipv4_only", "IPv4 only"),
    ("draft.families.ipv6_only", "IPv6 only"),
    ("draft.families.none", "No families selected"),
    ("location.region.europe", "Europe"),
    ("location.region.north_america_e", "North America East"),
    ("location.region.north_america_c", "North America Central"),
    ("location.region.north_america_w", "North America West"),
    ("location.region.central_america", "Central America"),
    ("location.region.south_america_e", "South America East"),
    ("location.region.south_america_w", "South America West"),
    ("location.region.africa_n", "North Africa"),
    ("location.region.africa_s", "Southern Africa"),
    ("location.region.asia_s", "South Asia"),
    ("location.region.asia_se", "Southeast Asia"),
    ("location.region.asia_e", "East Asia"),
    ("location.region.asia_n", "North Asia"),
    ("location.region.asia_w", "West Asia"),
    ("location.region.central_asia", "Central Asia"),
    ("location.region.pacific_oceania", "Pacific & Oceania"),
    ("location.region.antarctica", "Antarctica"),
    ("location.country.au", "Australia"),
    ("location.country.at", "Austria"),
    ("location.country.be", "Belgium"),
    ("location.country.br", "Brazil"),
    ("location.country.bg", "Bulgaria"),
    ("location.country.ca", "Canada"),
    ("location.country.cn", "China"),
    ("location.country.cz", "Czechia"),
    ("location.country.dk", "Denmark"),
    ("location.country.fi", "Finland"),
    ("location.country.fr", "France"),
    ("location.country.de", "Germany"),
    ("location.country.hk", "Hong Kong"),
    ("location.country.hu", "Hungary"),
    ("location.country.in", "India"),
    ("location.country.id", "Indonesia"),
    ("location.country.ie", "Ireland"),
    ("location.country.it", "Italy"),
    ("location.country.jp", "Japan"),
    ("location.country.kr", "South Korea"),
    ("location.country.lu", "Luxembourg"),
    ("location.country.my", "Malaysia"),
    ("location.country.nl", "Netherlands"),
    ("location.country.nz", "New Zealand"),
    ("location.country.no", "Norway"),
    ("location.country.pl", "Poland"),
    ("location.country.pt", "Portugal"),
    ("location.country.ro", "Romania"),
    ("location.country.ru", "Russia"),
    ("location.country.sg", "Singapore"),
    ("location.country.za", "South Africa"),
    ("location.country.es", "Spain"),
    ("location.country.se", "Sweden"),
    ("location.country.ch", "Switzerland"),
    ("location.country.tw", "Taiwan"),
    ("location.country.th", "Thailand"),
    ("location.country.tr", "Türkiye"),
    ("location.country.ua", "Ukraine"),
    ("location.country.gb", "United Kingdom"),
    ("location.country.us", "United States"),
    ("location.country.vn", "Vietnam"),
    ("location.direction.n", "North"),
    ("location.direction.s", "South"),
    ("location.direction.e", "East"),
    ("location.direction.w", "West"),
    ("location.direction.ne", "Northeast"),
    ("location.direction.nw", "Northwest"),
    ("location.direction.se", "Southeast"),
    ("location.direction.sw", "Southwest"),
    ("node.transport.ipv4", "IPv4"),
    ("node.transport.ipv6", "IPv6"),
    ("node.transport.dual_stack", "Dual-stack"),
    // Session / operation labels
    ("session_state.managed", "Managed"),
    ("session_state.manual", "Manual"),
    ("session_state.locked", "Locked"),
    ("session_state.pending_pr", "Pending PR"),
    ("session_state.stalled_pr", "Stalled PR"),
    ("session_state.conflict", "Conflict"),
    ("operation.kind.create", "Create"),
    ("operation.kind.update", "Update"),
    ("operation.kind.retire", "Retire"),
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
    (
        "operation.message.dropped",
        "Changes dropped — the pull request has been closed.",
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
    ("prompt.login", "login"),
    // Generic loading / errors
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
        "error.ui.session.locked",
        "This session has been locked and cannot be modified through autopeer",
    ),
    (
        "error.ui.session.choose_managed_to_retire",
        "Choose one of your sessions before you retire it",
    ),
    (
        "error.ui.session.choose_managed_to_delete",
        "Choose one of your sessions before you delete it",
    ),
    (
        "error.ui.auth.authenticate_first",
        "Authenticate before you continue",
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
        "error.runtime.decode_failed",
        "Failed to decode response: {detail}",
    ),
    (
        "error.runtime.http_failed",
        "HTTP request failed with status {status}",
    ),
    (
        "error.runtime.encode_failed",
        "Failed to encode payload: {detail}",
    ),
    (
        "error.runtime.unsupported_method",
        "Unsupported HTTP method {method}",
    ),
    ("error.runtime.request_failed", "Request failed: {detail}"),
    (
        "error.runtime.config.load_failed",
        "Failed to load config.json: {detail}",
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
        "error.runtime.config.autopeer_api_url.missing",
        "autopeer_api_url is not configured",
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
        "AS{asn} is invalid because it does not exist in the dn42 registry.",
    ),
    (
        "error.auth.asn.no_supported_auth",
        "AS{asn} exists in dn42, but it does not publish maintainer auth we can use yet.",
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
    (
        "error.request.operation.not_retryable",
        "This operation cannot be retried.",
    ),
    (
        "error.request.operation.not_droppable",
        "This operation cannot be dropped.",
    ),
    (
        "error.request.operation.pr_closed",
        "The pull request has been closed and cannot be retried.",
    ),
    (
        "error.request.operation.branch_missing",
        "The operation branch is missing from the repository.",
    ),
    ("error.request.route.not_found", "Not found."),
    (
        "error.vault.not_configured",
        "Vault encryption is not configured on this server. PSK and endpoint encryption are unavailable.",
    ),
    // Backend error messages
    (
        "error.repo.inventory.missing",
        "Network repo is missing inventory.yaml",
    ),
    (
        "error.repo.peer_file.missing",
        "Network repo is missing {path}",
    ),
    (
        "error.node.not_eligible",
        "Node {node} is not autopeer-eligible",
    ),
    (
        "error.node.not_accepting_changes",
        "{node} is not accepting autopeer changes right now",
    ),
    (
        "error.session.duplicate_on_node",
        "AS{asn} already has a session or pending operation on {node}",
    ),
    (
        "error.auth.asn.no_registry_auth.oidc_hint",
        "AS{asn} does not expose supported registry SSH, PGP, or email auth methods. Use one of the configured OIDC login options instead.",
    ),
    (
        "error.auth.impersonation.host_asn.cannot_mutate",
        "AS{asn} is one of our host ASN sessions; impersonate the ASN you want to manage before opening or modifying sessions",
    ),
    (
        "error.auth.impersonation.asn.not_host",
        "AS{asn} is not configured as a host ASN for impersonation",
    ),
    (
        "error.auth.registry_email.already_completed",
        "Registry email login has already completed; finish it from the emailed sign-in link.",
    ),
    (
        "error.request.session.mp_bgp_transport.invalid",
        "session.mp_bgp_transport must be one of ipv4, ipv6",
    ),
    (
        "error.request.session_payload.invalid",
        "Session payload is invalid",
    ),
    (
        "error.peer.duplicate",
        "Duplicate ASN AS{asn} exists in the peer file",
    ),
    (
        "error.peer.create.session_required",
        "Create operation requires a session payload",
    ),
    (
        "error.peer.managed.already_exists",
        "Managed peer AS{asn} already exists on this node",
    ),
    (
        "error.peer.not_found",
        "Peer AS{asn} does not exist on this node",
    ),
    (
        "error.peer.already_managed",
        "Peer AS{asn} is already managed by autopeer",
    ),
    (
        "error.peer.update.session_required",
        "Update operation requires a session payload",
    ),
    (
        "error.peer.manual.cannot_modify",
        "Manual peer AS{asn} cannot be modified by autopeer",
    ),
    (
        "error.peer.locked",
        "Peer AS{asn} is locked and cannot be modified through autopeer",
    ),
    (
        "error.data.yaml_root.invalid",
        "YAML root must be a mapping",
    ),
    (
        "error.data.peer_entry.invalid",
        "Peer entry must be a mapping",
    ),
    (
        "error.data.peer_entry.missing_bgp",
        "Peer entry is missing BGP mapping",
    ),
    (
        "error.data.peer_entry.missing_asn",
        "Peer entry is missing valid bgp.asn",
    ),
    (
        "error.data.peer.missing_wg",
        "Active peer AS{asn} is missing WireGuard mapping",
    ),
    (
        "error.data.peer_file.missing_peers",
        "Peer file must contain a top-level peers list",
    ),
    (
        "error.data.inventory.missing_all",
        "inventory.yaml is missing the top-level all key",
    ),
    (
        "error.data.inventory.missing_children",
        "inventory.yaml is missing all.children",
    ),
    (
        "error.data.inventory.missing_hosts",
        "inventory.yaml must define nodes.hosts and dn42.hosts",
    ),
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
        "validation.peer4.required_mp_bgp",
        "A peer IPv4 address is required for MP-BGP over IPv4 transport",
    ),
    (
        "validation.peer4.required_ipv4",
        "An IPv4 peer address is required for IPv4 routes",
    ),
    (
        "validation.peer6.required_mp_bgp",
        "A peer IPv6 address is required, or switch to IPv4 transport in Advanced options",
    ),
    (
        "validation.peer6.required_ipv6",
        "An IPv6 peer address is required for IPv6 routes",
    ),
    (
        "validation.peer6.required_enh",
        "An IPv6 peer address is required when ENH is enabled",
    ),
    (
        "validation.extended_next_hop.requires_mp_bgp",
        "Extended Next Hop requires MP-BGP",
    ),
    (
        "validation.extended_next_hop.requires_ipv4",
        "Extended Next Hop requires IPv4 routes",
    ),
    (
        "validation.extended_next_hop.requires_ipv6_transport",
        "Extended Next Hop requires IPv6 transport",
    ),
    (
        "validation.ipv4_over_ipv6_transport.requires_peer4_or_enh",
        "IPv4 over IPv6 transport requires a peer IPv4 address or Extended Next Hop",
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
    (
        "validation.own6.must_differ_from_peer6",
        "Peer link-local IPv6 must differ from our link-local IPv6",
    ),
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
        "validation.wg_public_key.charset",
        "Peer WireGuard key contains invalid base64 characters",
    ),
    (
        "validation.peer4.invalid",
        "Peer IPv4 address must be a valid IPv4 address",
    ),
    (
        "validation.peer4.range",
        "Peer IPv4 address must be a valid dn42 IPv4 address",
    ),
    (
        "validation.peer6.invalid",
        "Peer IPv6 address must be a valid IPv6 address",
    ),
    (
        "validation.peer6.scope",
        "Peer IPv6 address must be a valid dn42 ULA or link-local IPv6 address",
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
    (
        "validation.psk.length",
        "Pre-shared key must be a 44-character base64 key",
    ),
    (
        "validation.psk.charset",
        "Pre-shared key contains invalid base64 characters",
    ),
    // Backend-only validation
    (
        "validation.mp_bgp_transport.invalid",
        "MP-BGP transport must be one of ipv4, ipv6",
    ),
    (
        "validation.peering_strategy.invalid",
        "Peering strategy must be standard or aggressive",
    ),
    ("validation.port.range", "Port must be between 1 and 65535"),
    (
        "validation.endpoint.required",
        "Remote endpoint is required",
    ),
    (
        "validation.endpoint.node_ipv6_only",
        "{node} is IPv6-only; use a hostname or IPv6 endpoint",
    ),
    (
        "validation.endpoint.node_ipv4_only",
        "{node} is IPv4-only; use a hostname or IPv4 endpoint",
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
        "Fetching your dn42 registry authentication methods...",
    ),
    (
        "loading.redirect_oidc",
        "Redirecting you to your OIDC provider...",
    ),
    (
        "loading.fetch_challenge",
        "Fetching a fresh dn42 registry challenge for you...",
    ),
    (
        "loading.send_email",
        "Sending a sign-in link and one-time code to your registry email contacts...",
    ),
    ("loading.check_ssh", "Checking your SSH signature..."),
    ("loading.check_pgp", "Checking your PGP signature..."),
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
        "Authenticating the ASN against the dn42 registry...",
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
    (
        "operation.message.workflow_failed",
        "Workflow failed at {stage} stage ({conclusion})",
    ),
    (
        "operation.message.workflow_failed.step",
        "Workflow failed at {stage} stage, step \"{step}\" ({conclusion})",
    ),
    (
        "operation.message.workflow_failed.full",
        "Workflow failed at {stage} stage, step \"{step}\": {annotation} ({conclusion})",
    ),
    (
        "loading.delete_pr",
        "Deleting your session from our repo and opening a pull request...",
    ),
    (
        "loading.retry_operation",
        "Retrying your failed operation...",
    ),
    (
        "loading.drop_operation",
        "Dropping changes and closing the pull request...",
    ),
    // Worker validation / infrastructure errors
    ("error.field.required", "{field} is required."),
    ("error.field.must_be_string", "{field} must be a string."),
    ("error.field.must_be_boolean", "{field} must be a boolean."),
    ("error.field.must_be_integer", "{field} must be an integer."),
    ("error.field.must_be_object", "{field} must be an object."),
    ("error.asn.format", "ASN must look like 424242xxxx."),
    (
        "error.node.lock.unreadable",
        "Node lock for {node} could not be read.",
    ),
    (
        "error.email.send_failed",
        "Failed to send sign-in email: {detail}",
    ),
    (
        "error.github.api_failed",
        "GitHub request for {path} failed (HTTP {status}).",
    ),
    (
        "error.github.file_read_failed",
        "Failed to read {path} from GitHub (HTTP {status}).",
    ),
    (
        "error.registry.request_failed",
        "Registry request for {path} failed (HTTP {status}).",
    ),
    (
        "error.registry.invalid_payload",
        "Registry returned an unexpected payload for {path}.",
    ),
    (
        "error.registry.lookup_failed",
        "DN42 registry lookup failed while loading AS{asn}.",
    ),
    (
        "error.oidc.claim.asn_missing",
        "OIDC identity is missing the required ASN claim ({claim}).",
    ),
    (
        "error.oidc.claim.maintainer_missing",
        "OIDC identity is missing the required maintainer claim ({claim}).",
    ),
    (
        "error.oidc.maintainer.not_in_mnt_by",
        "{provider} asserted {candidates}, which is not in aut-num mnt-by.",
    ),
    (
        "error.oidc.discovery.failed",
        "OIDC discovery failed for {provider} (HTTP {status}).",
    ),
    (
        "error.oidc.discovery.invalid_json",
        "OIDC discovery for {provider} returned invalid JSON.",
    ),
    (
        "error.oidc.discovery.missing_field",
        "OIDC discovery for {provider} is missing {field}.",
    ),
    (
        "error.oidc.client_secret.missing",
        "{provider} is missing client_secret_env for {method}.",
    ),
    (
        "error.oidc.token.invalid_json",
        "{provider} returned invalid JSON from the token endpoint.",
    ),
    (
        "error.oidc.token.rejected",
        "{provider} rejected the login callback: {description}",
    ),
    (
        "error.oidc.userinfo.failed",
        "{provider} userinfo request failed (HTTP {status}).",
    ),
    (
        "error.oidc.userinfo.invalid_json",
        "{provider} userinfo endpoint returned invalid JSON.",
    ),
    (
        "error.oidc.id_token.missing",
        "{provider} did not return an ID token.",
    ),
    (
        "error.oidc.id_token.invalid",
        "{provider} ID token verification failed: {detail}",
    ),
    (
        "error.oidc.id_token.invalid_nonce",
        "{provider} returned a login token with an invalid nonce.",
    ),
    (
        "error.oidc.asn.mismatch",
        "OIDC identity ASN {token_asn} does not match requested ASN {requested_asn}.",
    ),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
