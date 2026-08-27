pub(super) const TABLE: &[(&str, &str)] = &[
    // App chrome
    ("app.title", "dn42 Autopeer"),
    ("app.title.footnote", "von IRIS-AS 4242421023"),
    ("nav.looking_glass", "Looking Glass"),
    ("nav.language", "Sprache"),
    // Generic actions
    ("action.back", "Zurück"),
    ("action.refresh", "Aktualisieren"),
    ("action.logout", "Abmelden"),
    ("action.cancel_edit", "Bearbeitung abbrechen"),
    ("action.choose_another_node", "Anderen Node wählen"),
    ("action.back_to_nodes", "Zurück zu den Nodes"),
    ("action.back_to_details", "Zurück zu den Details"),
    ("action.review_your_update", "Update prüfen"),
    ("action.review_your_change", "Änderung prüfen"),
    ("action.open_update_pr", "Update-PR öffnen"),
    ("action.open_create_pr", "Create-PR öffnen"),
    ("action.impersonate_this_asn", "Diese ASN impersonieren"),
    ("action.return_to_host_asn", "Zur Host-ASN zurückkehren"),
    ("action.confirm_retirement", "Stilllegung bestätigen"),
    ("action.retire_session", "Diese Session stilllegen"),
    ("action.confirm_deletion", "Löschung bestätigen"),
    ("action.delete_session", "Diese Session löschen"),
    ("action.open_pr", "PR öffnen"),
    ("action.workflow_run", "Workflow-Lauf"),
    ("action.retry", "Erneut versuchen"),
    ("action.redeploy", "Neu deployen"),
    ("action.drop_changes", "Änderungen verwerfen"),
    ("action.dismiss_operation", "Ausblenden"),
    ("action.check_bgp_session", "BGP-Sitzung prüfen"),
    // Step: LoadingConfig / AuthRedirect
    ("step.loading_config.prompt", "Wird geladen\u{2026}"),
    (
        "step.auth_redirect.prompt",
        "Authentifizieren Sie sich und verwalten Sie Ihre Peering-Sitzungen mit IRIS-AS 4242421023.",
    ),
    ("step.auth_redirect.link", "Bei dn42-auth.owo.li anmelden"),
    // Auth method labels returned by auth service sessions
    (
        "auth_method.registry_ssh.label",
        "Registrierung SSH Signatur",
    ),
    (
        "auth_method.registry_pgp.label",
        "Registrierung PGP Signatur",
    ),
    ("auth_method.registry_email.label", "Registrierungs-E-Mail"),
    (
        "auth_method.host_impersonation.label",
        "Host-ASN-Impersonation",
    ),
    // Manage / dashboard headings
    ("dashboard.flow_kicker", "Ihr Peering-Flow"),
    (
        "dashboard.host_readonly_title",
        "Sie sind als unsere Host-ASN angemeldet",
    ),
    ("dashboard.update_managed_title", "Session aktualisieren"),
    (
        "dashboard.create_or_manage_title",
        "Sessions erstellen oder verwalten",
    ),
    (
        "dashboard.host_readonly_body",
        "Unsere Host-ASN dient nur zur Unterstützung anderer Netzwerke. Bevor Sie Sessions erstellen, aktualisieren oder stilllegen, impersonieren Sie die ASN, die Sie verwalten möchten.",
    ),
    (
        "dashboard.create_or_manage_body",
        "Wählen Sie zunächst einen unserer Nodes. Danach können Sie eine neue Session erstellen oder eine bestehende öffnen, um sie zu aktualisieren oder stillzulegen.",
    ),
    ("dashboard.session_badge_template", "{mnt} über {label}"),
    // Sidebar
    ("sidebar.your_session_kicker", "Ihre Session"),
    ("sidebar.no_active_session", "Keine aktive Session"),
    (
        "sidebar.session_authed_template",
        "Sie haben sich über {label} als {mnt} authentifiziert.",
    ),
    ("sidebar.support_kicker", "Support-Modus"),
    ("sidebar.host_asn_prefix", "Host ASN AS{asn}"),
    (
        "sidebar.host_authed_template",
        "Sie haben sich über {label} als {mnt} authentifiziert. Dieser Modus dient nur dazu, Sessions im Namen einer anderen ASN zu verwalten.",
    ),
    ("sidebar.impersonate_asn_label", "impersonate_asn"),
    ("sidebar.effective_mnt_label", "effective_mnt"),
    ("sidebar.impersonate_asn_placeholder", "424242xxxx"),
    (
        "sidebar.impersonate_mnt_placeholder",
        "Optionale mntner-Überschreibung",
    ),
    ("sidebar.current_operation", "Aktueller Vorgang"),
    ("sidebar.support_mode_title", "Andere ASN impersonieren"),
    (
        "sidebar.support_mode_body",
        "Unsere Host-ASN dient nur zur Unterstützung anderer Netzwerke. Wählen Sie über das Impersonations-Formular die ASN, die Sie verwalten möchten.",
    ),
    // Stage 1: Select node
    ("stage1.kicker", "Stufe 1"),
    ("stage1.title", "Wählen Sie einen unserer Nodes"),
    ("flow.select_node.title", "Node wählen"),
    (
        "flow.select_node.description",
        "Der Node, der Ihrem Netzwerk am nächsten liegt, ist meist die beste Wahl.",
    ),
    ("flow.session_details.title", "Session konfigurieren"),
    (
        "flow.session_details.description",
        "Geben Sie WireGuard-Endpunkt und -Schlüssel, Tunneladressen und BGP-Optionen ein.",
    ),
    ("flow.review.title", "Überprüfen Sie Ihre Änderung"),
    (
        "flow.review.description",
        "Eine letzte Prüfung, bevor wir den Pull Request öffnen.",
    ),
    (
        "stage1.description",
        "Nodes ohne eigene Session lassen Sie eine neue erstellen. Nodes mit einer bestehenden Session öffnen diese zur Bearbeitung. Manuell angelegte Sessions werden beim Speichern in Autopeer übernommen. Nodes mit einer laufenden Änderung können erst nach deren Abschluss bearbeitet werden.",
    ),
    (
        "stage1.empty_title",
        "Wir haben keine Autopeer-fähigen Nodes für Ihre ASN gefunden.",
    ),
    (
        "stage1.empty_body",
        "Wenn das falsch aussieht, aktualisieren Sie die Seite oder überprüfen Sie unsere Autopeer-Richtlinie.",
    ),
    ("stage1.state.available", "Verfügbar"),
    ("stage1.state.disabled", "Deaktiviert"),
    (
        "stage1.state.note.create",
        "Erstellen Sie Ihre Session auf diesem Node.",
    ),
    (
        "stage1.state.note.managed",
        "Öffnen Sie diesen Node, um Ihre Session zu aktualisieren oder stillzulegen.",
    ),
    (
        "stage1.state.note.manual",
        "Öffnen Sie diesen Node, um die aktuelle Repo-Konfiguration zu prüfen. Beim Speichern wird die Session automatisch in Autopeer übernommen.",
    ),
    (
        "stage1.state.note.locked",
        "Diese Session wurde gesperrt und kann nicht über Autopeer geändert werden.",
    ),
    (
        "stage1.state.note.pending",
        "Eine Änderung für Ihre Session läuft hier bereits.",
    ),
    (
        "stage1.state.note.stalled",
        "Ein früheres Deployment ist fehlgeschlagen. Öffnen Sie diesen Node, um die Änderung zu bearbeiten und erneut einzureichen, das Deployment zu wiederholen oder die Änderung zu verwerfen.",
    ),
    (
        "stage1.state.note.conflict",
        "Dieser Node hat einen Konfigurationskonflikt, den unser Betreiber zuerst beheben muss.",
    ),
    (
        "stage1.state.note.disabled",
        "Dieser Node akzeptiert derzeit keine Autopeer-Sessions.",
    ),
    // Stalled PR banner
    ("stalled.banner.title", "Deployment fehlgeschlagen"),
    (
        "stalled.banner.body",
        "Eine frühere Änderung hat einen offenen PR, dessen Deployment fehlgeschlagen ist. Sie können die Konfiguration ändern und erneut einreichen, den vorhandenen PR erneut deployen oder die Änderungen vollständig verwerfen.",
    ),
    // Stage 2: Session details
    ("stage2.kicker", "Stufe 2"),
    (
        "stage2.title.update_prefix",
        "Session auf {node} aktualisieren oder stilllegen",
    ),
    (
        "stage2.title.create_prefix",
        "Session auf {node} einrichten",
    ),
    ("stage2.title.create_blank", "Neue Session einrichten"),
    (
        "stage2.update_intro",
        "Sie haben bereits eine verwaltete Session auf diesem Node. Aktualisieren Sie unten Ihre Peering-Details oder legen Sie die Session still, wenn Sie sie hier nicht mehr benötigen.",
    ),
    ("stage2.section.connection", "Verbindung"),
    ("stage2.section.tunnel", "Tunneladressen"),
    (
        "stage2.section.tunnel.help",
        "Verwenden Sie die Adressen, die Sie auf Ihrer Seite konfiguriert haben. IPv6 kann entweder ULA wie `fd42:...` oder link-lokal wie `fe80:...` sein.",
    ),
    ("stage2.section.families", "Routenfamilien"),
    (
        "stage2.section.families.help",
        "Wählen Sie aus, welche dn42-Routenfamilien Ihre Sitzung übertragen soll.",
    ),
    ("stage2.section.bgp", "BGP Verhalten"),
    (
        "stage2.section.bgp.help",
        "MP-BGP überträgt IPv4- und IPv6-Routen über eine einzelne BGP-Session auf dem von Ihnen gewählten Transport. Wenn Sie es deaktivieren, richten wir für jede Routenfamilie eine eigene BGP-Session ein. Extended Next Hop erlaubt es der Session, IPv4-Routen über IPv6-Transport ohne IPv4-Tunneladresse zu übertragen.",
    ),
    ("stage2.section.policy", "Routing-Richtlinie"),
    ("stage2.advanced.summary", "Erweiterte Optionen"),
    ("stage2.field.endpoint", "Endpunkt"),
    (
        "stage2.field.endpoint.placeholder",
        "Hostname oder IP:Port Ihres Routers",
    ),
    ("stage2.field.wg_key", "WireGuard-Schlüssel"),
    (
        "stage2.field.wg_key.placeholder",
        "Base64-Public-Key Ihres Routers",
    ),
    ("stage2.field.peer4", "Peer-IPv4-Adresse"),
    (
        "stage2.field.peer4.placeholder",
        "Ihre dn42 IPv4 Adresse, z.B. 172.21.111.111",
    ),
    ("stage2.field.peer6", "Peer-IPv6-Adresse"),
    (
        "stage2.field.peer6.placeholder",
        "Ihr ULA oder Link-Local, z.B. fd42:4242:1023:: oder fe80::",
    ),
    ("stage2.field.own6_link_local", "Unser Link-Local IPv6"),
    (
        "stage2.field.own6_link_local.placeholder",
        "Wird nur benötigt, wenn Ihre Peer-Adresse IPv6 verbindungslokal ist",
    ),
    ("stage2.field.own6_node", "Unsere Node-IPv6"),
    (
        "stage2.field.own6_node.no_inventory",
        "Dieser Node hat keine IPv6-Adresse.",
    ),
    ("stage2.field.own4_node", "Unsere Node-IPv4"),
    (
        "stage2.field.own4_node.no_inventory",
        "Dieser Node hat keine IPv4-Adresse.",
    ),
    ("stage2.field.families", "Familien"),
    ("stage2.field.families.ipv4_label", "IPv4 Routen"),
    ("stage2.field.families.ipv6_label", "IPv6 Routen"),
    ("stage2.field.bgp_features", "Merkmale"),
    ("stage2.field.bgp.mpbgp_label", "MP-BGP"),
    ("stage2.field.bgp.enh_label", "Extended Next Hop"),
    ("stage2.field.bgp.transport", "Transport"),
    ("stage2.field.policy", "Politik"),
    ("stage2.field.comment", "Kommentar"),
    (
        "stage2.field.comment.placeholder",
        "Optionale Notiz zu Ihrer Session",
    ),
    ("stage2.field.keepalive", "Dauerhaftes Keepalive"),
    (
        "stage2.field.keepalive.placeholder",
        "Optionales Keepalive in Sekunden für Ihren Router",
    ),
    ("stage2.field.mtu", "Schnittstelle MTU"),
    ("stage2.field.mtu.placeholder", "Optional MTU"),
    ("stage2.field.psk", "Pre-Shared Key"),
    ("stage2.field.psk.placeholder", "Optionaler WireGuard-PSK"),
    (
        "stage2.field.psk.placeholder.existing",
        "PSK konfiguriert, zum Beibehalten leer lassen",
    ),
    ("stage2.field.psk.clear", "PSK löschen"),
    ("stage2.field.psk.generate", "PSK erzeugen"),
    ("stage2.field.psk.copied", "Kopiert"),
    (
        "stage2.field.psk.help",
        "Ein optionaler WireGuard Pre-Shared Key für zusätzliche Sicherheit. Der Schlüssel wird vor dem Speichern verschlüsselt.",
    ),
    ("stage2.field.encrypt_endpoint", "Verschlüsselt"),
    (
        "stage2.field.encrypt_endpoint.help",
        "Verschlüsseln Sie Ihre Endpunktadresse im Git-Repo, sodass sie nicht im Klartext sichtbar ist.",
    ),
    // Stage 3: Review
    ("stage3.kicker", "Stufe 3"),
    ("stage3.title", "Bestätigen Sie Ihre Session-Details"),
    ("stage3.review.our_node", "Unser Node"),
    ("stage3.review.not_selected", "Nicht ausgewählt"),
    ("stage3.review.endpoint", "Endpunkt"),
    ("stage3.review.wg_key", "WireGuard-Public-Key"),
    ("stage3.review.route_families", "Routenfamilien"),
    ("stage3.review.bgp_behavior", "BGP Verhalten"),
    ("stage3.review.bgp.mpbgp", "MP-BGP"),
    ("stage3.review.bgp.separate", "Separate IPv4/IPv6-Sitzungen"),
    ("stage3.review.bgp.enh_suffix", " + Extended Next Hop"),
    ("stage3.review.routing_policy", "Routing-Richtlinie"),
    ("stage3.review.peer4", "Peer-IPv4-Adresse"),
    ("stage3.review.peer6", "Peer-IPv6-Adresse"),
    ("stage3.review.own6", "Unsere Link-Local-IPv6"),
    ("stage3.review.keepalive", "Dauerhaftes Keepalive"),
    ("stage3.review.mtu", "MTU"),
    ("stage3.review.psk", "Pre-Shared Key"),
    ("stage3.review.psk.set", "Konfiguriert (verschlüsselt)"),
    ("stage3.review.psk.not_set", "Nicht festgelegt"),
    ("stage3.review.psk.unchanged", "Konfiguriert (unverändert)"),
    ("stage3.review.psk.cleared", "Wird entfernt"),
    ("stage3.review.encrypt_endpoint.enabled", "Verschlüsselt"),
    ("stage3.review.note", "Ihre Notiz"),
    ("stage3.review.our_node_details", "Unsere Node-Details"),
    ("stage3.review.our_endpoint", "Endpunkt"),
    ("stage3.review.our_ipv4", "IPv4"),
    ("stage3.review.our_ipv6", "IPv6"),
    ("stage3.review.our_link_local_ipv6", "Link-Local-IPv6"),
    ("stage3.review.our_wg_pubkey", "WireGuard-Public-Key"),
    ("stage3.review.our_node_note", "Notiz"),
    ("stage3.review.check_session_label", "BGP-Sitzung"),
    // Draft / node formatting
    ("draft.families.ipv4_ipv6", "IPv4 + IPv6"),
    ("draft.families.ipv4_only", "Nur IPv4"),
    ("draft.families.ipv6_only", "Nur IPv6"),
    ("draft.families.none", "Keine Familien ausgewählt"),
    ("location.region.europe", "Europa"),
    ("location.region.north_america_e", "Nordamerika Ost"),
    ("location.region.north_america_c", "Nordamerika Zentral"),
    ("location.region.north_america_w", "Nordamerika West"),
    ("location.region.central_america", "Mittelamerika"),
    ("location.region.south_america_e", "Südamerika Ost"),
    ("location.region.south_america_w", "Südamerika Westen"),
    ("location.region.africa_n", "Nordafrika"),
    ("location.region.africa_s", "Südliches Afrika"),
    ("location.region.asia_s", "Südasien"),
    ("location.region.asia_se", "Südostasien"),
    ("location.region.asia_e", "Ostasien"),
    ("location.region.asia_n", "Nordasien"),
    ("location.region.asia_w", "Westasien"),
    ("location.region.central_asia", "Zentralasien"),
    ("location.region.pacific_oceania", "Pazifik und Ozeanien"),
    ("location.region.antarctica", "Antarktis"),
    ("location.country.au", "Australien"),
    ("location.country.at", "Österreich"),
    ("location.country.be", "Belgien"),
    ("location.country.br", "Brasilien"),
    ("location.country.bg", "Bulgarien"),
    ("location.country.ca", "Kanada"),
    ("location.country.cn", "China"),
    ("location.country.cz", "Tschechien"),
    ("location.country.dk", "Dänemark"),
    ("location.country.fi", "Finnland"),
    ("location.country.fr", "Frankreich"),
    ("location.country.de", "Deutschland"),
    ("location.country.hk", "Hongkong"),
    ("location.country.hu", "Ungarn"),
    ("location.country.in", "Indien"),
    ("location.country.id", "Indonesien"),
    ("location.country.ie", "Irland"),
    ("location.country.it", "Italien"),
    ("location.country.jp", "Japan"),
    ("location.country.kr", "Südkorea"),
    ("location.country.lu", "Luxemburg"),
    ("location.country.my", "Malaysia"),
    ("location.country.nl", "Niederlande"),
    ("location.country.nz", "Neuseeland"),
    ("location.country.no", "Norwegen"),
    ("location.country.pl", "Polen"),
    ("location.country.pt", "Portugal"),
    ("location.country.ro", "Rumänien"),
    ("location.country.ru", "Russland"),
    ("location.country.sg", "Singapur"),
    ("location.country.za", "Südafrika"),
    ("location.country.es", "Spanien"),
    ("location.country.se", "Schweden"),
    ("location.country.ch", "Schweiz"),
    ("location.country.tw", "Taiwan"),
    ("location.country.th", "Thailand"),
    ("location.country.tr", "Türkei"),
    ("location.country.ua", "Ukraine"),
    ("location.country.gb", "Vereinigtes Königreich"),
    ("location.country.us", "Vereinigte Staaten"),
    ("location.country.vn", "Vietnam"),
    ("location.direction.n", "Norden"),
    ("location.direction.s", "Süden"),
    ("location.direction.e", "Ost"),
    ("location.direction.w", "Westen"),
    ("location.direction.ne", "Nordost"),
    ("location.direction.nw", "Nordwest"),
    ("location.direction.se", "Südost"),
    ("location.direction.sw", "Südwesten"),
    ("node.transport.ipv4", "IPv4"),
    ("node.transport.ipv6", "IPv6"),
    ("node.transport.dual_stack", "Dual-Stack"),
    // Session / operation labels
    ("session_state.managed", "Verwaltet"),
    ("session_state.manual", "Manuell"),
    ("session_state.locked", "Gesperrt"),
    ("session_state.pending_pr", "Ausstehender PR"),
    ("session_state.stalled_pr", "Steckengebliebener PR"),
    ("session_state.conflict", "Konflikt"),
    ("operation.kind.create", "Erstellen"),
    ("operation.kind.update", "Aktualisieren"),
    ("operation.kind.retire", "Stilllegen"),
    ("operation.kind.delete", "Löschen"),
    ("operation.kind.migrate", "Wandern"),
    (
        "operation.state.pending_pull_request",
        "PR wird vorbereitet",
    ),
    ("operation.state.pending_checks", "Warten auf CI"),
    ("operation.state.applying", "Apply auf Node"),
    ("operation.state.pending_merge", "Warten auf Merge"),
    ("operation.state.completed", "Vollendet"),
    ("operation.state.failed", "Fehlgeschlagen"),
    ("operation.state.conflict", "Konflikt"),
    // Backend operation messages
    (
        "operation.message.pending_pull_request",
        "Wir bereiten Ihren Pull Request vor.",
    ),
    (
        "operation.message.pending_checks",
        "Ihr Pull Request ist offen; wir warten auf peer-session-check.",
    ),
    (
        "operation.message.applying",
        "Checks bestanden; Ihre Session wird zur Verifikation auf dem Node angewendet.",
    ),
    (
        "operation.message.pending_merge",
        "Apply auf dem Node erfolgreich; wir warten auf den Merge.",
    ),
    (
        "operation.message.completed",
        "Ihre Änderung wurde erfolgreich übernommen und zusammengeführt.",
    ),
    (
        "operation.message.failed",
        "Ihre Änderung ist fehlgeschlagen.",
    ),
    (
        "operation.message.conflict",
        "Wir konnten Ihre Änderung wegen eines Konfigurationskonflikts auf unserer Seite nicht anwenden.",
    ),
    (
        "operation.message.wait_node_lock",
        "Apply erfolgreich; wir warten, bis eine andere Änderung auf diesem Node fertig gemergt ist.",
    ),
    (
        "operation.message.no_change",
        "Ihre Session entspricht bereits unserem Repo, daher haben wir keinen Pull Request geöffnet.",
    ),
    (
        "operation.message.check_not_started",
        "peer-session-check wurde für Ihre Pull Request nicht gestartet.",
    ),
    (
        "operation.message.check_wait_start",
        "Ihre Pull Request ist offen; Warten auf den Start von peer-session-check.",
    ),
    (
        "operation.message.check_failed",
        "peer-session-check endete mit {conclusion}.",
    ),
    (
        "operation.message.apply_not_started",
        "peer-session-apply wurde für Ihre Pull Request nicht gestartet.",
    ),
    (
        "operation.message.apply_wait_start",
        "Checks bestanden; Warten auf den Start von peer-session-apply.",
    ),
    (
        "operation.message.apply_failed",
        "peer-session-apply endete mit {conclusion}.",
    ),
    (
        "operation.message.pull_request_closed",
        "Ihre Pull Request wurde vor der Zusammenführung geschlossen.",
    ),
    (
        "operation.message.merge_failed",
        "Warten auf Zusammenführung. Zusammenführungsversuch fehlgeschlagen: {error}",
    ),
    (
        "operation.message.dropped",
        "Änderungen verworfen. Der Pull-Request wurde geschlossen.",
    ),
    ("operation.failure_stage.checks", "CI prüft"),
    ("operation.failure_stage.preflight", "Node-Preflight"),
    ("operation.failure_stage.apply", "Node-Apply"),
    ("operation.failure_stage.merge", "Verschmelzen"),
    // Routing policy labels
    ("peering_strategy.full_table.label", "Full Table"),
    (
        "peering_strategy.full_table.description",
        "Wir nehmen alle gültigen Routen von Ihnen an und senden Ihnen alle gültigen Routen.",
    ),
    ("peering_strategy.transit.label", "Transit"),
    (
        "peering_strategy.transit.description",
        "Wir nehmen alle gültigen Routen von Ihnen an und senden Ihnen nur unsere eigenen Präfixe.",
    ),
    ("peering_strategy.peer.label", "Peer"),
    (
        "peering_strategy.peer.description",
        "Wir nehmen nur Ihre direkten Routen an und senden Ihnen unsere eigenen Präfixe sowie Routen unserer Downstreams.",
    ),
    ("peering_strategy.downstream.label", "Downstream"),
    (
        "peering_strategy.downstream.description",
        "Wir nehmen nur Ihre direkten Routen an und senden Ihnen alle gültigen Routen.",
    ),
    // Operation progress labels
    ("operation.progress.branch", "Zweig"),
    ("operation.progress.checks", "Checks"),
    ("operation.progress.apply", "Apply"),
    ("operation.progress.merge", "Verschmelzen"),
    ("operation.progress.done", "Erledigt"),
    // Operation failure labels
    ("operation.failure.stage", "Fehlgeschlagene Phase"),
    ("operation.failure.conclusion", "Ergebnis"),
    // Prompts (shell-style left labels)
    ("prompt.autopeer", "autopeer"),
    ("prompt.login", "login"),
    // Generic loading / errors
    (
        "error.ui.node.choose",
        "Wählen Sie einen unserer Nodes, bevor Sie fortfahren",
    ),
    (
        "error.ui.node.choose_inline",
        "Wählen Sie einen unserer Nodes, bevor Sie einen PR öffnen",
    ),
    (
        "error.ui.session.missing_config",
        "Ihrer aktuellen Session fehlen Konfigurationsdetails",
    ),
    (
        "error.ui.operation.wait_inflight",
        "Auf diesem Node läuft bereits eine Änderung. Warten Sie zuerst, bis sie abgeschlossen ist.",
    ),
    (
        "error.ui.node.blocked_conflict",
        "Dieser Node ist durch einen Konflikt in unserem Repo blockiert",
    ),
    (
        "error.ui.session.locked",
        "Diese Session wurde gesperrt und kann nicht über Autopeer geändert werden",
    ),
    (
        "error.ui.session.choose_managed_to_retire",
        "Wählen Sie eine Ihrer Sessions aus, bevor Sie sie stilllegen",
    ),
    (
        "error.ui.session.choose_managed_to_delete",
        "Wählen Sie eine Ihrer Sessions aus, bevor Sie sie löschen",
    ),
    (
        "error.ui.auth.authenticate_first",
        "Authentifizieren Sie sich, bevor Sie fortfahren",
    ),
    ("error.auth.asn.required", "ASN ist erforderlich"),
    (
        "error.auth.oidc.provider.missing",
        "Der Anbieter OIDC fehlt",
    ),
    ("error.request.challenge_id.missing", "challenge_id fehlt"),
    (
        "error.ui.auth.method.choose_first",
        "Wählen Sie zunächst eine Authentifizierungsmethode.",
    ),
    (
        "error.ui.auth.registry_email.inactive",
        "Die Registrierungs-E-Mail-Authentifizierung ist derzeit nicht aktiv.",
    ),
    (
        "error.ui.auth.registry_email.choose_maintainer",
        "Wählen Sie zunächst einen Betreuer mit Registrierungs-E-Mail-Kontakten aus.",
    ),
    (
        "error.ui.auth.registry_email.code.required",
        "Geben Sie den einmaligen Authentifizierungscode aus Ihrer E-Mail ein.",
    ),
    (
        "error.auth.method.unavailable",
        "Diese Authentifizierungsmethode ist für Ihre ASN nicht mehr verfügbar.",
    ),
    (
        "error.ui.auth.impersonation.host_auth_first",
        "Authentifizieren Sie eine unserer konfigurierten Host-ASNs, bevor Sie sich als eine andere ASN ausgeben.",
    ),
    (
        "error.ui.auth.impersonation.asn.required",
        "Geben Sie die ASN ein, die Sie imitieren möchten",
    ),
    (
        "error.ui.auth.impersonation.host_session.missing",
        "Derzeit ist keine Host-ASN-Session verfügbar",
    ),
    (
        "error.ui.auth.impersonation.host_required",
        "Der Identitätswechsel ist erst verfügbar, nachdem Sie eine unserer konfigurierten Host-ASNs authentifiziert haben.",
    ),
    (
        "error.runtime.decode_failed",
        "Antwort konnte nicht dekodiert werden: {detail}",
    ),
    (
        "error.runtime.http_failed",
        "HTTP-Anfrage mit Status {status} fehlgeschlagen",
    ),
    (
        "error.runtime.encode_failed",
        "Nutzlast konnte nicht kodiert werden: {detail}",
    ),
    (
        "error.runtime.unsupported_method",
        "Nicht unterstützte HTTP-Methode {method}",
    ),
    (
        "error.runtime.request_failed",
        "Anfrage fehlgeschlagen: {detail}",
    ),
    (
        "error.runtime.config.load_failed",
        "config.json konnte nicht geladen werden: {detail}",
    ),
    (
        "error.runtime.browser.unavailable",
        "Das Browserfenster ist nicht verfügbar",
    ),
    (
        "error.runtime.oidc.redirect_failed",
        "Die OIDC-Anmeldeumleitung konnte nicht geöffnet werden",
    ),
    (
        "error.runtime.config.autopeer_api_url.missing",
        "autopeer_api_url ist nicht konfiguriert",
    ),
    (
        "error.auth.ssh.empty_or_missing_blocks",
        "Fügen Sie den vollständig abgetrennten SSH-Signaturblock aus dem obigen Befehl ein, einschließlich der BEGIN/END-Zeilen.",
    ),
    (
        "error.auth.ssh.unsigned_challenge",
        "Fügen Sie den abgetrennten SSH-Signaturblock aus dem obigen Befehl ein, nicht den unsignierten Challenge-Text.",
    ),
    (
        "error.request.body.invalid_json",
        "Der Request-Body muss gültiges JSON sein.",
    ),
    (
        "error.auth.asn.unsupported",
        "Autopeer unterstützt derzeit nur ASNs im Bereich 424242xxxx.",
    ),
    (
        "error.auth.asn.not_found",
        "AS{asn} existiert nicht in der dn42-Registrierung.",
    ),
    (
        "error.auth.asn.no_supported_auth",
        "AS{asn} existiert in dn42, veröffentlicht aber noch keine Betreuer-Authentifizierung, die wir verwenden können.",
    ),
    (
        "error.auth.registry_email.unavailable",
        "Die Registrierungs-E-Mail-Anmeldung ist in dieser Bereitstellung nicht verfügbar.",
    ),
    ("error.auth.challenge.unknown_id", "Unbekannt challenge_id."),
    (
        "error.auth.challenge.expired",
        "Ihre Authentifizierungsherausforderung ist abgelaufen.",
    ),
    (
        "error.auth.challenge.used",
        "Diese Authentifizierungsherausforderung wurde bereits verwendet.",
    ),
    (
        "error.auth.session.token.missing",
        "Bearer-Session-Token fehlt.",
    ),
    (
        "error.auth.session.unknown",
        "Unbekannte Authentifizierungssitzung.",
    ),
    (
        "error.auth.session.expired",
        "Die Authentifizierungssitzung ist abgelaufen.",
    ),
    (
        "error.auth.impersonation.no_maintainers",
        "Für diesen ASN stehen keine Betreuer zum Identitätswechsel zur Verfügung.",
    ),
    (
        "error.auth.ssh.malformed_signature",
        "Die SSH-Signaturdaten sind fehlerhaft. Führen Sie ssh-keygen -Y sign erneut aus und fügen Sie den vollständigen abgetrennten Signaturblock ein.",
    ),
    (
        "error.auth.ssh.unrecognized_key",
        "Ihre SSH-Signatur wurde mit einem Schlüssel erstellt, der in keinem Betreuer-Objekt (mntner) dieser ASN aufgeführt ist.",
    ),
    (
        "error.auth.ssh.verification_failed",
        "SSH-Signaturprüfung fehlgeschlagen.",
    ),
    (
        "error.auth.pgp.invalid_public_key",
        "Der öffentliche PGP-Schlüssel ist ungültig. Exportieren Sie Ihren ASCII-armored Public Key und fügen Sie den vollständigen Block ein.",
    ),
    (
        "error.auth.pgp.invalid_signed_message",
        "Die PGP-signierte Nachricht ist ungültig. Clear-signieren Sie die Challenge und fügen Sie den vollständigen signierten Block ein.",
    ),
    (
        "error.auth.pgp.verification_failed",
        "PGP-Signaturprüfung fehlgeschlagen. Signieren Sie die Challenge erneut mit dem passenden Registry-Schlüssel und fügen Sie den vollständigen signierten Block ein.",
    ),
    (
        "error.auth.pgp.unrecognized_key",
        "Ihr PGP-Fingerabdruck {fingerprint} ist in keinem Betreuer-Objekt (mntner) dieser ASN aufgeführt.",
    ),
    (
        "error.auth.pgp.challenge_mismatch",
        "Ihre PGP-signierte Nachricht stimmt nicht mit der ausgegebenen Challenge überein.",
    ),
    (
        "error.auth.registry_email.state.missing",
        "Der Registrierungs-E-Mail-Anmeldestatus wurde nicht gefunden oder ist abgelaufen.",
    ),
    (
        "error.auth.registry_email.state.expired",
        "Die Registrierungs-E-Mail-Anmeldung ist abgelaufen.",
    ),
    (
        "error.auth.registry_email.state.pending",
        "Die Registrierungs-E-Mail-Anmeldung ist noch nicht abgeschlossen.",
    ),
    (
        "error.auth.registry_email.code.invalid",
        "Der E-Mail-Authentifizierungscode der Registrierung ist ungültig.",
    ),
    (
        "error.auth.registry_email.session.missing",
        "Die Registrierungs-E-Mail-Anmeldesitzung ist nicht mehr verfügbar.",
    ),
    (
        "error.auth.registry_email.session.expired",
        "Die E-Mail-Anmeldesitzung für die Registrierung ist abgelaufen.",
    ),
    (
        "error.auth.registry_email.callback.params.missing",
        "Es fehlen Parameter für den E-Mail-Rückruf der Registrierung.",
    ),
    (
        "error.auth.registry_email.callback.failed",
        "Die Registrierung per E-Mail ist fehlgeschlagen. Bitte versuchen Sie es noch einmal.",
    ),
    (
        "error.auth.registry_email.contacts.missing",
        "AS{asn} stellt keine admin-c- oder tech-c-E-Mail-Adressen zur Verfügung, die wir in der Registrierung verwenden können.",
    ),
    (
        "error.auth.registry_email.target.missing",
        "{requested} verfügt nicht über Registrierungs-E-Mail-Kontakte, die wir für diesen ASN verwenden können.",
    ),
    (
        "error.auth.registry_email.target.required",
        "effective_mnt ist erforderlich, wenn Ihre Registrierungs-E-Mail-Authentifizierung mehrere Betreuer abdeckt.",
    ),
    (
        "error.auth.oidc.callback.provider.missing",
        "Der Anbieter OIDC fehlt im Rückrufpfad.",
    ),
    (
        "error.auth.oidc.callback.params.missing",
        "Fehlende OIDC-Rückrufparameter.",
    ),
    (
        "error.auth.oidc.provider.unknown",
        "Unbekannter OIDC-Anbieter {provider}.",
    ),
    (
        "error.auth.oidc.provider.rejected",
        "{error}: {description}",
    ),
    (
        "error.auth.oidc.state.missing",
        "Der Anmeldestatus OIDC wurde nicht gefunden oder ist abgelaufen.",
    ),
    (
        "error.auth.oidc.state.expired",
        "Der Anmeldestatus von OIDC ist abgelaufen.",
    ),
    (
        "error.auth.oidc.state.pending",
        "Die Anmeldung von OIDC ist noch nicht abgeschlossen.",
    ),
    (
        "error.auth.oidc.session.missing",
        "OIDC-Anmeldesitzung ist nicht mehr verfügbar.",
    ),
    (
        "error.auth.oidc.session.expired",
        "OIDC Anmeldesitzung ist abgelaufen.",
    ),
    (
        "error.auth.oidc.callback.failed",
        "OIDC Anmeldung fehlgeschlagen; Bitte versuchen Sie es noch einmal.",
    ),
    (
        "error.auth.oidc.identity.asn_mismatch",
        "OIDC Identität ASN {token_asn} stimmt nicht mit der angeforderten ASN {requested_asn} überein.",
    ),
    (
        "error.auth.session.path_asn_mismatch",
        "Die ASN im Pfad stimmt nicht mit Ihrer authentifizierten Session überein.",
    ),
    ("error.request.node.required", "Node ist erforderlich."),
    (
        "error.request.session_payload.required",
        "Session-Payload ist erforderlich.",
    ),
    (
        "error.auth.impersonation.maintainer.required",
        "effective_mnt ist erforderlich, wenn Ihr Ziel ASN mehrere Betreuer hat. Verfügbare Mntner: {available}.",
    ),
    (
        "error.auth.impersonation.maintainer.missing",
        "{requested} ist in aut-num -> mnt-by für diese ASN nicht vorhanden. Verfügbare Mntner: {available}.",
    ),
    (
        "error.request.operation.not_found",
        "Vorgang nicht gefunden.",
    ),
    (
        "error.request.operation.not_retryable",
        "Dieser Vorgang kann nicht wiederholt werden.",
    ),
    (
        "error.request.operation.not_droppable",
        "Dieser Vorgang kann nicht gelöscht werden.",
    ),
    (
        "error.request.operation.pr_closed",
        "Die Pull Request wurde geschlossen und kann nicht erneut versucht werden.",
    ),
    (
        "error.request.operation.branch_missing",
        "Der Operationszweig fehlt im Repo.",
    ),
    ("error.request.route.not_found", "Nicht gefunden."),
    (
        "error.vault.not_configured",
        "Die Tresorverschlüsselung ist auf diesem Server nicht konfiguriert. PSK und Endpunktverschlüsselung sind nicht verfügbar.",
    ),
    // Backend error messages
    (
        "error.repo.inventory.missing",
        "Netzwerk-Repo fehlt inventory.yaml",
    ),
    ("error.repo.peer_file.missing", "Netzwerk-Repo fehlt {path}"),
    (
        "error.node.not_eligible",
        "Knoten {node} ist nicht für Autopeer geeignet",
    ),
    (
        "error.node.not_accepting_changes",
        "{node} akzeptiert derzeit keine Autopeer-Änderungen",
    ),
    (
        "error.session.duplicate_on_node",
        "AS{asn} hat bereits eine Sitzung oder einen ausstehenden Vorgang auf {node}",
    ),
    (
        "error.auth.asn.no_registry_auth.oidc_hint",
        "AS{asn} stellt keine unterstützten Registrierungsmethoden SSH, PGP oder E-Mail-Authentifizierung bereit. Verwenden Sie stattdessen eine der konfigurierten OIDC-Anmeldeoptionen.",
    ),
    (
        "error.auth.impersonation.host_asn.cannot_mutate",
        "Sie sind als AS{asn} angemeldet, einer unserer Host-ASNs. Impersonieren Sie die ASN, die Sie verwalten möchten, bevor Sie Sessions erstellen oder ändern.",
    ),
    (
        "error.auth.impersonation.asn.not_host",
        "AS{asn} ist nicht als Host ASN für den Identitätswechsel konfiguriert",
    ),
    (
        "error.auth.registry_email.already_completed",
        "Die Registrierungs-E-Mail-Anmeldung ist bereits abgeschlossen. Beenden Sie den Vorgang über den per E-Mail gesendeten Anmeldelink.",
    ),
    (
        "error.request.session.mp_bgp_transport.invalid",
        "session.mp_bgp_transport muss entweder IPv4 oder IPv6 sein",
    ),
    (
        "error.request.session_payload.invalid",
        "Session-Payload ist ungültig",
    ),
    (
        "error.peer.duplicate",
        "In der Peer-Datei ist ein Duplikat von ASN AS{asn} vorhanden",
    ),
    (
        "error.peer.create.session_required",
        "Create-Vorgang erfordert einen Session-Payload",
    ),
    (
        "error.peer.managed.already_exists",
        "Verwalteter Peer AS{asn} existiert auf diesem Node bereits",
    ),
    (
        "error.peer.not_found",
        "Peer AS{asn} existiert auf diesem Node nicht",
    ),
    (
        "error.peer.already_managed",
        "Peer AS{asn} wird bereits von Autopeer verwaltet",
    ),
    (
        "error.peer.update.session_required",
        "Update-Vorgang erfordert einen Session-Payload",
    ),
    (
        "error.peer.manual.cannot_modify",
        "Der manuelle Peer AS{asn} kann nicht vom Autopeer geändert werden",
    ),
    (
        "error.peer.locked",
        "Peer AS{asn} ist gesperrt und kann nicht über Autopeer geändert werden",
    ),
    (
        "error.data.yaml_root.invalid",
        "YAML root muss eine Zuordnung sein",
    ),
    (
        "error.data.peer_entry.invalid",
        "Der Peer-Eintrag muss eine Zuordnung sein",
    ),
    (
        "error.data.peer_entry.missing_bgp",
        "Beim Peer-Eintrag fehlt die BGP-Zuordnung",
    ),
    (
        "error.data.peer_entry.missing_asn",
        "Peer-Eintrag fehlt gültig bgp.asn",
    ),
    (
        "error.data.peer.missing_wg",
        "Dem aktiven Peer AS{asn} fehlt die WireGuard-Zuordnung",
    ),
    (
        "error.data.peer_file.missing_peers",
        "Die Peer-Datei muss eine Peers-Liste der obersten Ebene enthalten",
    ),
    (
        "error.data.inventory.missing_all",
        "inventory.yaml fehlt der All-Schlüssel der obersten Ebene",
    ),
    (
        "error.data.inventory.missing_children",
        "inventory.yaml fehlen alle Kinder",
    ),
    (
        "error.data.inventory.missing_hosts",
        "inventory.yaml muss nodes.hosts und dn42.hosts definieren",
    ),
    // Frontend validation
    (
        "validation.tunnel.required",
        "Fügen Sie mindestens eine Tunneladresse hinzu: IPv4 oder IPv6",
    ),
    (
        "validation.bgp_family.required",
        "Aktivieren Sie mindestens eine BGP-Familie",
    ),
    (
        "validation.peer4.required_mp_bgp",
        "Für den Transport von MP-BGP über IPv4 ist eine Peer-Adresse IPv4 erforderlich",
    ),
    (
        "validation.peer4.required_ipv4",
        "Für IPv4-Routen ist eine IPv4-Peer-Adresse erforderlich",
    ),
    (
        "validation.peer6.required_mp_bgp",
        "Eine Peer-Adresse IPv6 ist erforderlich, oder wechseln Sie in den erweiterten Optionen zum Transport IPv4",
    ),
    (
        "validation.peer6.required_ipv6",
        "Für IPv6-Routen ist eine IPv6-Peer-Adresse erforderlich",
    ),
    (
        "validation.peer6.required_enh",
        "Wenn ENH aktiviert ist, ist eine IPv6-Peer-Adresse erforderlich",
    ),
    (
        "validation.extended_next_hop.requires_mp_bgp",
        "Extended Next Hop erfordert MP-BGP",
    ),
    (
        "validation.extended_next_hop.requires_ipv4",
        "Extended Next Hop erfordert IPv4 Routen",
    ),
    (
        "validation.extended_next_hop.requires_ipv6_transport",
        "Extended Next Hop erfordert den Transport IPv6",
    ),
    (
        "validation.ipv4_over_ipv6_transport.requires_peer4_or_enh",
        "Der Transport von IPv4 über IPv6 erfordert eine Peer-Adresse IPv4 oder Extended Next Hop.",
    ),
    (
        "validation.own6.requires_peer6",
        "Eine lokale Link-Local-Adresse IPv6 benötigt eine Peer-Adresse IPv6",
    ),
    (
        "validation.own6.requires_link_local_peer6",
        "Lokaler Link-Local IPv6 gilt nur, wenn die Peer-Adresse IPv6 Link-Local ist",
    ),
    (
        "validation.own6.must_start_fe80",
        "Local link-local IPv6 muss mit fe80 beginnen:",
    ),
    (
        "validation.own6.must_differ_from_peer6",
        "Peer-Link-Local IPv6 muss sich von unserem Link-Local IPv6 unterscheiden",
    ),
    (
        "validation.endpoint.no_spaces",
        "Der Remote-Endpunkt darf keine Leerzeichen enthalten",
    ),
    (
        "validation.endpoint.ipv6_format",
        "IPv6-Endpunkte müssen das Format [addr]:port verwenden",
    ),
    (
        "validation.endpoint.ipv6_invalid",
        "Die IPv6-Adresse des Remote-Endpunkts muss eine gültige IPv6-Adresse sein",
    ),
    (
        "validation.endpoint.host_port_format",
        "Remote-Endpunkt muss host:port oder [ipv6]:port verwenden",
    ),
    (
        "validation.endpoint.port_required",
        "Der Remote-Endpunkt muss einen Port enthalten",
    ),
    (
        "validation.endpoint.host_required",
        "Remote-Endpunkt-Host ist erforderlich",
    ),
    (
        "validation.endpoint.host_invalid",
        "Der Host des Remote-Endpunkts muss eine IPv4-Adresse oder ein vollständig qualifizierter Hostname sein",
    ),
    (
        "validation.endpoint.port.invalid",
        "Der Remote-Endpunkt-Port muss eine gültige Nummer sein",
    ),
    (
        "validation.endpoint.port.range",
        "Der Remote-Endpunkt-Port muss zwischen 1 und 65535 liegen",
    ),
    (
        "validation.wg_public_key.required",
        "wg_public_key ist erforderlich",
    ),
    (
        "validation.wg_public_key.length",
        "Der Peer-WireGuard-Schlüssel muss ein öffentlicher Base64-Schlüssel mit 44 Zeichen sein",
    ),
    (
        "validation.wg_public_key.charset",
        "Peer-Schlüssel WireGuard enthält ungültige Base64-Zeichen",
    ),
    (
        "validation.peer4.invalid",
        "Die Peer-Adresse IPv4 muss eine gültige Adresse IPv4 sein",
    ),
    (
        "validation.peer4.range",
        "Die Peer-Adresse IPv4 muss eine gültige Adresse dn42 IPv4 sein",
    ),
    (
        "validation.peer6.invalid",
        "Die Peer-Adresse IPv6 muss eine gültige Adresse IPv6 sein",
    ),
    (
        "validation.peer6.scope",
        "Die Peer-Adresse IPv6 muss eine gültige Adresse dn42 ULA oder eine verbindungslokale Adresse IPv6 sein",
    ),
    (
        "validation.own6.invalid",
        "Local link-local IPv6 muss eine gültige IPv6-Adresse sein",
    ),
    (
        "validation.own6.scope",
        "Die lokale Link-Local-Adresse IPv6 muss eine Link-Local-Adresse IPv6 sein",
    ),
    (
        "validation.keepalive.invalid",
        "Persistentes Keepalive muss eine gültige Zahl sein",
    ),
    (
        "validation.mtu.invalid",
        "Schnittstelle MTU muss eine gültige Nummer sein",
    ),
    (
        "validation.mtu.range",
        "Schnittstelle MTU muss zwischen 1280 und 1500 liegen",
    ),
    (
        "validation.psk.length",
        "Der Pre-Shared Key muss ein 44 Zeichen langer Base64-Schlüssel sein",
    ),
    (
        "validation.psk.charset",
        "Der Pre-Shared Key enthält ungültige Base64-Zeichen",
    ),
    // Backend-only validation
    (
        "validation.mp_bgp_transport.invalid",
        "Der MP-BGP-Transport muss entweder IPv4 oder IPv6 sein",
    ),
    (
        "validation.peering_strategy.invalid",
        "Die Peering-Strategie muss Standard oder aggressiv sein",
    ),
    (
        "validation.port.range",
        "Der Port muss zwischen 1 und 65535 liegen",
    ),
    (
        "validation.endpoint.required",
        "Remote-Endpunkt ist erforderlich",
    ),
    (
        "validation.endpoint.node_ipv6_only",
        "{node} ist nur IPv6; Verwenden Sie einen Hostnamen oder einen IPv6-Endpunkt",
    ),
    (
        "validation.endpoint.node_ipv4_only",
        "{node} ist nur IPv4; Verwenden Sie einen Hostnamen oder einen IPv4-Endpunkt",
    ),
    // Loading messages
    (
        "loading.email_login",
        "E-Mail-Login wird abgeschlossen und Ihre Sessions werden geladen...",
    ),
    (
        "loading.oidc_login",
        "OIDC-Login wird abgeschlossen und Ihre Sessions werden geladen...",
    ),
    (
        "loading.fetch_sessions",
        "Ihre aktuellen Sessions werden aus unserem Repo geladen...",
    ),
    (
        "loading.refresh_sessions",
        "Session-Status wird aus unserem Repo aktualisiert...",
    ),
    (
        "loading.fetch_methods",
        "Ihre dn42-Registry-Authentifizierungsmethoden werden geladen...",
    ),
    (
        "loading.redirect_oidc",
        "Sie werden zu Ihrem OIDC-Anbieter weitergeleitet...",
    ),
    (
        "loading.fetch_challenge",
        "Holen Sie sich eine neue dn42-Registrierungsherausforderung für Sie ...",
    ),
    (
        "loading.send_email",
        "Anmeldelink und Einmalcode werden an Ihre Registry-E-Mail-Kontakte gesendet...",
    ),
    ("loading.check_ssh", "Überprüfen Sie Ihre SSH-Signatur ..."),
    ("loading.check_pgp", "Überprüfen Sie Ihre PGP-Signatur ..."),
    (
        "loading.check_email",
        "Überprüfen Sie den E-Mail-Authentifizierungscode Ihrer Registrierung ...",
    ),
    (
        "loading.host_session_prep",
        "Host-ASN-Session wird vorbereitet...",
    ),
    (
        "loading.authing_asn",
        "Authentifizieren von ASN anhand der Registrierung dn42 ...",
    ),
    (
        "loading.restore_host",
        "Host-ASN-Session wird aus unserem Repo wiederhergestellt...",
    ),
    (
        "loading.update_pr",
        "Peering-Konfiguration wird in unserem Repo aktualisiert und ein Pull Request geöffnet...",
    ),
    (
        "loading.create_pr",
        "Peering-Konfiguration wird in unserem Repo erstellt und ein Pull Request geöffnet...",
    ),
    (
        "loading.retire_pr",
        "Session wird in unserem Repo stillgelegt und ein Pull Request geöffnet...",
    ),
    (
        "operation.message.workflow_failed",
        "Der Workflow ist in der Phase {stage} ({conclusion}) fehlgeschlagen.",
    ),
    (
        "operation.message.workflow_failed.step",
        "Der Workflow ist in der Phase {stage}, Schritt „{step}“ ({conclusion}) fehlgeschlagen.",
    ),
    (
        "operation.message.workflow_failed.full",
        "Der Workflow ist in der Phase {stage}, Schritt „{step}“ fehlgeschlagen: {annotation} ({conclusion})",
    ),
    (
        "loading.delete_pr",
        "Session wird aus unserem Repo gelöscht und ein Pull Request geöffnet...",
    ),
    (
        "loading.retry_operation",
        "Der fehlgeschlagene Vorgang wird erneut ausgeführt...",
    ),
    (
        "loading.drop_operation",
        "Änderungen verwerfen und Pull-Request schließen...",
    ),
    // Worker validation / infrastructure errors
    ("error.field.required", "{field} ist erforderlich."),
    (
        "error.field.must_be_string",
        "{field} muss eine Zeichenkette sein.",
    ),
    (
        "error.field.must_be_boolean",
        "{field} muss ein Boolescher Wert sein.",
    ),
    (
        "error.field.must_be_integer",
        "{field} muss eine ganze Zahl sein.",
    ),
    (
        "error.field.must_be_object",
        "{field} muss ein Objekt sein.",
    ),
    (
        "error.asn.format",
        "Die ASN muss dem Format 424242xxxx entsprechen.",
    ),
    (
        "error.node.lock.unreadable",
        "Knotensperre für {node} konnte nicht gelesen werden.",
    ),
    (
        "error.email.send_failed",
        "Anmelde-E-Mail konnte nicht gesendet werden: {detail}",
    ),
    (
        "error.github.api_failed",
        "GitHub-Anfrage für {path} fehlgeschlagen (HTTP {status}).",
    ),
    (
        "error.github.file_read_failed",
        "Lesen von {path} aus GitHub fehlgeschlagen (HTTP {status}).",
    ),
    (
        "error.registry.request_failed",
        "Registry-Anfrage für {path} fehlgeschlagen (HTTP {status}).",
    ),
    (
        "error.registry.invalid_payload",
        "Die Registry hat eine unerwartete Antwort für {path} zurückgegeben.",
    ),
    (
        "error.registry.lookup_failed",
        "DN42-Registry-Lookup für AS{asn} fehlgeschlagen.",
    ),
    (
        "error.oidc.claim.asn_missing",
        "Der OIDC-Identität fehlt der erforderliche ASN-Claim ({claim}).",
    ),
    (
        "error.oidc.claim.maintainer_missing",
        "Der OIDC-Identität fehlt der erforderliche Maintainer-Claim ({claim}).",
    ),
    (
        "error.oidc.maintainer.not_in_mnt_by",
        "{provider} hat {candidates} bestätigt, was nicht in aut-num mnt-by enthalten ist.",
    ),
    (
        "error.oidc.discovery.failed",
        "OIDC-Discovery für {provider} fehlgeschlagen (HTTP {status}).",
    ),
    (
        "error.oidc.discovery.invalid_json",
        "OIDC-Discovery für {provider} hat ungültiges JSON zurückgegeben.",
    ),
    (
        "error.oidc.discovery.missing_field",
        "Bei OIDC-Discovery für {provider} fehlt {field}.",
    ),
    (
        "error.oidc.client_secret.missing",
        "{provider} fehlt client_secret_env für {method}.",
    ),
    (
        "error.oidc.token.invalid_json",
        "{provider} hat ungültiges JSON vom Token-Endpunkt zurückgegeben.",
    ),
    (
        "error.oidc.token.rejected",
        "{provider} hat den Login-Callback abgelehnt: {description}",
    ),
    (
        "error.oidc.userinfo.failed",
        "{provider}-Userinfo-Anfrage fehlgeschlagen (HTTP {status}).",
    ),
    (
        "error.oidc.userinfo.invalid_json",
        "{provider}-Userinfo-Endpunkt hat ungültiges JSON zurückgegeben.",
    ),
    (
        "error.oidc.id_token.missing",
        "{provider} hat kein ID-Token zurückgegeben.",
    ),
    (
        "error.oidc.id_token.invalid",
        "{provider}-ID-Token-Verifizierung fehlgeschlagen: {detail}",
    ),
    (
        "error.oidc.id_token.invalid_nonce",
        "{provider} hat ein Login-Token mit ungültigem Nonce zurückgegeben.",
    ),
    (
        "error.oidc.asn.mismatch",
        "OIDC-Identitäts-ASN {token_asn} stimmt nicht mit der angeforderten ASN {requested_asn} überein.",
    ),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
