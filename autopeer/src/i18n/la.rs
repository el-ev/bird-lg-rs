pub(super) const TABLE: &[(&str, &str)] = &[
    // App chrome
    ("app.title", "dn42 Autopeer"),
    ("app.title.footnote", "ex IRIS-AS 4242421023"),
    ("nav.looking_glass", "Speculum"),
    ("nav.language", "Lingua"),
    // Generic actions
    ("action.back", "Retro"),
    ("action.refresh", "Renovare"),
    ("action.logout", "Exire"),
    ("action.cancel_edit", "Editionem Cancellare"),
    ("action.choose_another_node", "Alium Nodum Eligere"),
    ("action.back_to_nodes", "Ad Nodos Reverti"),
    ("action.back_to_details", "Ad Singula Reverti"),
    ("action.review_your_update", "Renovationem Tuam Recensere"),
    ("action.review_your_change", "Mutationem Tuam Recensere"),
    ("action.open_update_pr", "PR Renovationis Aperire"),
    ("action.open_create_pr", "PR Creationis Aperire"),
    ("action.impersonate_this_asn", "Hunc ASN Personare"),
    ("action.return_to_host_asn", "Ad ASN Hospitis Reverti"),
    ("action.find_registry_auth", "Modos Auctoritatis Registri Invenire"),
    ("action.verify", "Verificare"),
    ("action.verify_code", "Codicem Verificare"),
    ("action.send_signin_link", "Nexum Subscriptionis Mittere"),
    ("action.resend_signin_link", "Nexum Subscriptionis Remittere"),
    ("action.confirm_retirement", "Retractum Confirmare"),
    ("action.retire_session", "Hanc Sessionem Retrahere"),
    ("action.confirm_deletion", "Deletionem Confirmare"),
    ("action.delete_session", "Hanc Sessionem Delere"),
    ("action.open_pr", "PR Aperire"),
    ("action.workflow_run", "Workflow Currere"),
    ("action.retry", "Iterare"),
    ("action.dismiss_operation", "Dimittere"),
    // Step: LoadingConfig / EnterAsn
    (
        "step.loading_config.prompt",
        "Configurationem temporis exsequendi oneratur",
    ),
    (
        "step.loading_config.message",
        "Configurationem temporis exsequendi oneratur...",
    ),
    (
        "step.enter_asn.prompt",
        "Inscribe ASN tuum dn42 pro auctoritate SSH, PGP, vel electronica.",
    ),
    ("step.enter_asn.placeholder", "424242xxxx"),
    (
        "step.enter_asn.oidc_alt",
        "Vel intra cum provisore identitatis tuae et nos ASN tuum automatice deducemus.",
    ),
    ("step.enter_asn.continue_with", "Pergere cum {provider}"),
    // Step: SelectMethod
    (
        "step.select_method.found_for_as",
        "Modos auctoritatis registri invenimus pro AS{asn}",
    ),
    // Backend auth method copy
    ("auth_method.registry_ssh.label", "Subscriptio SSH Registri"),
    (
        "auth_method.registry_ssh.description",
        "Signa provocationem nostram cum clave SSH ex obiecto custodis tui dn42.",
    ),
    ("auth_method.registry_pgp.label", "Subscriptio PGP Registri"),
    (
        "auth_method.registry_pgp.description",
        "Utere una ex vestigiis digitalibus PGP registri tui: {fingerprints}",
    ),
    ("auth_method.registry_email.label", "Electronica Registri"),
    (
        "auth_method.registry_email.description",
        "Elige custodem et mitte nexum subscriptionis ad inscriptiones electronicas registri eius.",
    ),
    (
        "auth_method.registry_email.description_single",
        "Mitte nexum subscriptionis et codicem unius usus ad {emails}.",
    ),
    (
        "auth_method.registry_ssh.session_description",
        "Cum {mnt} auctoritate SSH autenticatus es.",
    ),
    (
        "auth_method.registry_pgp.session_description",
        "Cum {mnt} auctoritate PGP autenticatus es.",
    ),
    (
        "auth_method.registry_email.session_description",
        "Cum {mnt} auctoritate electronica autenticatus es.",
    ),
    (
        "auth_method.host_impersonation.label",
        "Personatio ASN Hospitis",
    ),
    (
        "auth_method.host_impersonation.description",
        "{mnt} per ASN hospitis nostri AS{host_asn} personas.",
    ),
    (
        "auth_method.oidc.description",
        "Autenticare cum {provider} et unam ex petitionibus custodis tui pro hoc ASN probare.",
    ),
    (
        "auth_method.oidc.session_description",
        "Cum {provider} ut {mnt} autenticatus es.",
    ),
    // Step: VerifyMethod (SSH)
    (
        "verify.ssh.no_fingerprints",
        "Vestigia digitalia clavium SSH pro ASN tuo invenire non potuimus.",
    ),
    ("verify.ssh.match_one", "Clavem SSH tuam {fingerprint} conferre"),
    (
        "verify.ssh.match_many",
        "Unam ex clavibus SSH tuis conferre: {fingerprints}",
    ),
    ("verify.ssh.create_signature", "Provocationem signare"),
    (
        "verify.ssh.paste_prompt",
        "Mandatum supra exsequere, deinde subscriptionem SSH seiunctam tuam appone.",
    ),
    ("verify.ssh.placeholder", "-----BEGIN SSH SIGNATURE-----"),
    // Step: VerifyMethod (PGP)
    (
        "verify.pgp.no_fingerprints",
        "Vestigia digitalia PGP pro ASN tuo invenire non potuimus",
    ),
    ("verify.pgp.use_key", "Clave tua {fingerprint} utere"),
    (
        "verify.pgp.clearsign_intro",
        "Textum provocationis exactum cum clave tua congruente clare signa, deinde eandem clavem publicam exporta et utrumque exitum infra appone.",
    ),
    ("verify.pgp.exact_challenge", "Textus provocationis"),
    ("verify.pgp.clearsign_label", "Provocationem tuam clare signa"),
    (
        "verify.pgp.signed_paste_prompt",
        "Provocationem tuam clare signatam ex mandato supra appone",
    ),
    (
        "verify.pgp.signed_placeholder",
        "-----BEGIN PGP SIGNED MESSAGE-----",
    ),
    ("verify.pgp.export_label", "Clavem publicam tuam exporta"),
    (
        "verify.pgp.pubkey_paste_prompt",
        "Clavem publicam tuam loricatam ASCII ex mandato exportationis supra appone",
    ),
    (
        "verify.pgp.pubkey_placeholder",
        "-----BEGIN PGP PUBLIC KEY BLOCK-----",
    ),
    // Step: VerifyMethod (Email)
    (
        "verify.email.intro",
        "Mitte nexum subscriptionis et codicem unius usus ad inscriptiones electronicas registri unius ex custodibus tuis, deinde nexum preme vel codicem infra appone.",
    ),
    (
        "verify.email.no_contacts",
        "Inscriptiones electronicas admin-c vel tech-c pro ASN tuo invenire non potuimus.",
    ),
    ("verify.email.auth_as", "Autenticare ut {mnt}"),
    ("verify.email.send_to", "Mittere ad {emails}"),
    (
        "verify.email.sent_to_prefix",
        "Nexum subscriptionis et codicem auctoritatis ad {emails} misimus.",
    ),
    (
        "verify.email.code_prompt",
        "Codicem auctoritatis ex electronica tua appone",
    ),
    ("verify.email.code_placeholder", "12345678"),
    // Step: VerifyMethod (OIDC / Host)
    ("verify.oidc.continue_to", "Pergere ad {provider}"),
    (
        "verify.oidc.in_browser",
        "Pergere ad {provider} in navigatro tuo",
    ),
    (
        "verify.oidc.redirect_note",
        "Te ad provisorem tuum transferemus, deinde huc reducimus postquam ASN et petitiones custodis tui probaverit.",
    ),
    (
        "verify.host.note",
        "Personatio praesto est postquam unum ex ASN-ibus hospitis configuratis nostris autenticaveris.",
    ),
    (
        "verify.choose_first",
        "Prius modum autenticationis elige.",
    ),
    ("verify.auth_for_as", "{label} pro AS{asn}"),
    // Manage / dashboard headings
    ("dashboard.flow_kicker", "Fluxus Peering Tuus"),
    (
        "dashboard.host_readonly_title",
        "ASN hospitis noster hic tantum legi potest",
    ),
    ("dashboard.update_managed_title", "Sessionem tuam renovare"),
    (
        "dashboard.create_or_manage_title",
        "Sessiones tuas creare vel administrare",
    ),
    (
        "dashboard.host_readonly_body",
        "ASN hospitis noster solum ad alias retia adiuvanda est. Antequam sessiones crees, renoves, vel retrahas, ASN quam administrare vis persona.",
    ),
    (
        "dashboard.create_or_manage_body",
        "Semel autenticare et unum ex nodis nostris elige. Inde sessionem novam creare potes, vel sessionem existentem aperire ad renovationem vel retractionem.",
    ),
    ("dashboard.session_badge_template", "{mnt} per {label}"),
    // Sidebar
    ("sidebar.your_session_kicker", "Sessio Tua"),
    ("sidebar.no_active_session", "Nulla sessio activa"),
    (
        "sidebar.session_authed_template",
        "Ut {mnt} per {label} autenticatus es.",
    ),
    ("sidebar.support_kicker", "Modus Auxilii"),
    ("sidebar.host_asn_prefix", "ASN Hospitis AS{asn}"),
    (
        "sidebar.host_authed_template",
        "Ut {mnt} per {label} autenticatus es. Hoc solum utere cum sessiones pro alio ASN aperire vel reparare debes.",
    ),
    ("sidebar.impersonate_asn_label", "impersonate_asn"),
    ("sidebar.effective_mnt_label", "effective_mnt"),
    ("sidebar.impersonate_asn_placeholder", "424242xxxx"),
    (
        "sidebar.impersonate_mnt_placeholder",
        "Praescriptio mntner optionalis",
    ),
    ("sidebar.current_operation", "Operatio Praesens"),
    ("sidebar.support_mode_title", "Alium ASN personare"),
    (
        "sidebar.support_mode_body",
        "Hic ASN hospitis solum ad alias retia adiuvandum est. Utere organis ad dextram ad ASN quam administrare vis personandum.",
    ),
    // Stage 1: Select node
    ("stage1.kicker", "Gradus I"),
    ("stage1.title", "Unum ex nodis nostris elige"),
    ("flow.select_node.title", "Nodum Eligere"),
    (
        "flow.select_node.description",
        "Nodum proximum in rete nostro elige antequam singula cuniculi compleas.",
    ),
    ("flow.session_details.title", "Sessionem Tuam Configurare"),
    (
        "flow.session_details.description",
        "Valores WireGuard et BGP inscribe, deinde optiones provectas quas opus habes adiusta.",
    ),
    ("flow.review.title", "Mutationem Tuam Recensere"),
    (
        "flow.review.description",
        "Mutationem tuam recense antequam petitionem tractionis aperiamus.",
    ),
    (
        "stage1.description",
        "Nodum in rete nostro elige. Nodi vacui sessiones novas creari sinunt; sessiones existentes in loco aperiuntur pro renovationibus. Sessiones manuales in autopeer automatice adoptantur cum servas. Nodi in transitu tantum legi possunt.",
    ),
    (
        "stage1.empty_title",
        "Nullos nodos autopeer-habilitatos pro ASN tuo invenimus.",
    ),
    (
        "stage1.empty_body",
        "Si id falsum videtur, paginam renova vel consilium autopeer nostrum inspice.",
    ),
    ("stage1.state.available", "Praesto"),
    ("stage1.state.disabled", "Inhibitus"),
    (
        "stage1.state.note.create",
        "Sessionem tuam in hoc nodo crea.",
    ),
    (
        "stage1.state.note.managed",
        "Hunc nodum aperi ad sessionem tuam renovandaṃ vel retrahendaṃ.",
    ),
    (
        "stage1.state.note.manual",
        "Hunc nodum aperi ad configurationem repositorii currentem recensendam. Servando eam sessionem in autopeer automatice adoptabis.",
    ),
    (
        "stage1.state.note.pending",
        "Mutatio pro sessione tua iam hic in progressu est.",
    ),
    (
        "stage1.state.note.conflict",
        "Repositorium nostrum in conflictu pro hoc nodo est.",
    ),
    (
        "stage1.state.note.disabled",
        "Hic nodus sessiones autopeer nunc non accipit.",
    ),
    // Stage 2: Session details
    ("stage2.kicker", "Gradus II"),
    (
        "stage2.title.update_prefix",
        "Sessionem tuam in {node} renovare vel retrahere",
    ),
    (
        "stage2.title.create_prefix",
        "Sessionem tuam in {node} instituere",
    ),
    ("stage2.title.create_blank", "Sessionem novam tuam instituere"),
    (
        "stage2.update_intro",
        "Sessionem administratam in hoc nodo iam habes. Singula peering tua infra renova, vel sessionem retrahe si eam hic amplius non vis.",
    ),
    ("stage2.section.connection", "Connexio"),
    ("stage2.section.tunnel", "Inscriptiones Cuniculi"),
    (
        "stage2.section.tunnel.help",
        "Utere inscriptionibus quas in parte tua configurasti. IPv6 potest esse ULA ut `fd42:...` vel nexus localis ut `fe80:...`.",
    ),
    ("stage2.section.families", "Familiae Itinerum"),
    (
        "stage2.section.families.help",
        "Elige quas familias itinerum dn42 sessio tua portare debet.",
    ),
    ("stage2.section.bgp", "Comportamentum BGP"),
    (
        "stage2.section.bgp.help",
        "MP-BGP unam sessionem BGP super transitu IPv4 vel IPv6 selecto tuo utitur ad itinera IPv4 et/vel IPv6 portanda; si eam inhibes, sessiones BGP separatas generabimus, et Saltus Proximus Extensus solum ad itinera IPv4 super transitu IPv6 portata spectat.",
    ),
    ("stage2.section.policy", "Consilium Cursus"),
    ("stage2.advanced.summary", "Optiones provectae"),
    ("stage2.field.endpoint", "Terminus"),
    (
        "stage2.field.endpoint.placeholder",
        "Nomen vel IP:portus cursitoris tui",
    ),
    ("stage2.field.wg_key", "Clavis WireGuard"),
    (
        "stage2.field.wg_key.placeholder",
        "Clavis publica Base64 ex cursitore tuo",
    ),
    ("stage2.field.peer4", "Inscriptio IPv4 paris"),
    (
        "stage2.field.peer4.placeholder",
        "Inscriptio tua IPv4 dn42, e.g. 172.21.111.111",
    ),
    ("stage2.field.peer6", "Inscriptio IPv6 paris"),
    (
        "stage2.field.peer6.placeholder",
        "ULA vel nexus localis tuus, e.g. fd42:4242:1023:: vel fe80::",
    ),
    ("stage2.field.own6_link_local", "IPv6 nexus localis noster"),
    (
        "stage2.field.own6_link_local.placeholder",
        "Solum necessarium cum inscriptio IPv6 paris nexus localis est",
    ),
    ("stage2.field.own6_node", "IPv6 nodi nostri"),
    (
        "stage2.field.own6_node.no_inventory",
        "Inventarium nostrum inscriptionem IPv6 pro hoc nodo non enumerat.",
    ),
    ("stage2.field.own4_node", "IPv4 nodi nostri"),
    (
        "stage2.field.own4_node.no_inventory",
        "Inventarium nostrum inscriptionem IPv4 pro hoc nodo non enumerat.",
    ),
    ("stage2.field.families", "Familiae"),
    ("stage2.field.families.ipv4_label", "Itinera IPv4"),
    ("stage2.field.families.ipv6_label", "Itinera IPv6"),
    ("stage2.field.bgp_features", "Proprietates"),
    ("stage2.field.bgp.mpbgp_label", "MP-BGP"),
    ("stage2.field.bgp.enh_label", "Saltus Proximus Extensus"),
    ("stage2.field.bgp.transport", "Transitus"),
    ("stage2.field.policy", "Consilium"),
    ("stage2.field.comment", "Commentarium"),
    (
        "stage2.field.comment.placeholder",
        "Nota optionalis de sessione tua",
    ),
    ("stage2.field.keepalive", "Custodia perpetua"),
    (
        "stage2.field.keepalive.placeholder",
        "Custodia optionalis in secundis pro cursitore tuo",
    ),
    ("stage2.field.mtu", "MTU interface"),
    ("stage2.field.mtu.placeholder", "MTU optionalis"),
    ("stage2.field.psk", "Clavis praedivisa"),
    ("stage2.field.psk.placeholder", "Clavis WireGuard PSK optionalis (base64)"),
    ("stage2.field.psk.help", "Clavis praedivisa WireGuard optionalis ad securitatem augendam. Clavis ante repositionem encryptabitur."),
    ("stage2.field.encrypt_endpoint", "Encryptum"),
    ("stage2.field.encrypt_endpoint.help", "Encrypta inscriptionem terminationis in repositorio git ne in textu aperto appareat."),
    // Stage 3: Review
    ("stage3.kicker", "Gradus III"),
    ("stage3.title", "Mutationem tuam recense antequam PR aperiamus"),
    ("stage3.review.our_node", "Nodus noster"),
    ("stage3.review.not_selected", "Non selectus"),
    ("stage3.review.endpoint", "Terminus"),
    ("stage3.review.wg_key", "Clavis publica WireGuard"),
    ("stage3.review.route_families", "Familiae itinerum"),
    ("stage3.review.bgp_behavior", "Comportamentum BGP"),
    ("stage3.review.bgp.mpbgp", "MP-BGP"),
    ("stage3.review.bgp.separate", "Sessiones IPv4/IPv6 separatae"),
    ("stage3.review.bgp.enh_suffix", " + Saltus Proximus Extensus"),
    ("stage3.review.routing_policy", "Consilium cursus"),
    ("stage3.review.peer4", "Inscriptio IPv4 paris"),
    ("stage3.review.peer6", "Inscriptio IPv6 paris"),
    ("stage3.review.own6", "IPv6 nexus localis noster"),
    ("stage3.review.keepalive", "Custodia perpetua"),
    ("stage3.review.mtu", "MTU"),
    ("stage3.review.psk", "Clavis praedivisa"),
    ("stage3.review.psk.set", "Configurata (encrypta)"),
    ("stage3.review.psk.not_set", "Non posita"),
    ("stage3.review.encrypt_endpoint", "Encryptio terminationis"),
    ("stage3.review.encrypt_endpoint.enabled", "Activata"),
    ("stage3.review.encrypt_endpoint.disabled", "Inactivata"),
    ("stage3.review.note", "Nota tua"),
    ("stage3.review.our_node_details", "Singula nodi nostri"),
    ("stage3.review.our_endpoint", "Terminus"),
    ("stage3.review.our_ipv4", "IPv4"),
    ("stage3.review.our_ipv6", "IPv6"),
    ("stage3.review.our_link_local_ipv6", "IPv6 nexus localis"),
    ("stage3.review.our_wg_pubkey", "Clavis publica WireGuard"),
    ("stage3.review.our_node_note", "Nota"),
    // Draft / node formatting
    ("draft.families.ipv4_ipv6", "IPv4 + IPv6"),
    ("draft.families.ipv4_only", "IPv4 solum"),
    ("draft.families.ipv6_only", "IPv6 solum"),
    ("draft.families.none", "Nullae familiae selectae"),
    ("location.direction.n", "Septentrionalis"),
    ("location.direction.s", "Meridionalis"),
    ("location.direction.e", "Orientalis"),
    ("location.direction.w", "Occidentalis"),
    ("location.direction.ne", "Septentrionalis Orientalis"),
    ("location.direction.nw", "Septentrionalis Occidentalis"),
    ("location.direction.se", "Meridionalis Orientalis"),
    ("location.direction.sw", "Meridionalis Occidentalis"),
    ("node.transport.ipv4", "IPv4"),
    ("node.transport.ipv6", "IPv6"),
    ("node.transport.dual_stack", "Duplex"),
    // Session / operation labels
    ("session_state.managed", "Administratus"),
    ("session_state.manual", "Manualis"),
    ("session_state.pending_pr", "PR Expectans"),
    ("session_state.conflict", "Conflictus"),
    ("session.badge.psk", "PSK"),
    ("session.badge.encrypted_endpoint", "Terminatio encrypta"),
    ("operation.kind.create", "Creare"),
    ("operation.kind.update", "Renovare"),
    ("operation.kind.retire", "Retrahere"),
    ("operation.kind.delete", "Delere"),
    ("operation.kind.migrate", "Migrare"),
    ("operation.state.pending_pull_request", "PR Paratur"),
    ("operation.state.pending_checks", "CI Expectatur"),
    ("operation.state.applying", "In Nodo Applicatur"),
    ("operation.state.pending_merge", "Fusio Expectatur"),
    ("operation.state.completed", "Perfectum"),
    ("operation.state.failed", "Defectum"),
    ("operation.state.conflict", "Conflictus"),
    // Backend operation messages
    (
        "operation.message.pending_pull_request",
        "Petitionem tractionis tuam paramus.",
    ),
    (
        "operation.message.pending_checks",
        "Petitio tractionis tua aperta est; peer-session-check expectatur.",
    ),
    (
        "operation.message.applying",
        "Probationes superatae; sessionem tuam nodo ad verificationem applicamus.",
    ),
    (
        "operation.message.pending_merge",
        "Applicatio in nodo successit; fusio expectatur.",
    ),
    (
        "operation.message.completed",
        "Mutatio tua applicata et fusa est feliciter.",
    ),
    ("operation.message.failed", "Mutatio tua defecit."),
    (
        "operation.message.conflict",
        "Mutationem tuam applicare non potuimus quia repositorium nostrum in conflictu erat.",
    ),
    (
        "operation.message.wait_node_lock",
        "Applicatio successit; alia mutatio in hoc nodo fundi expectatur.",
    ),
    (
        "operation.message.no_change",
        "Sessio tua repositorium nostrum iam congruebat, ita petitionem tractionis non aperuimus.",
    ),
    (
        "operation.message.check_not_started",
        "peer-session-check pro petitione tractionis tua non incepit.",
    ),
    (
        "operation.message.check_wait_start",
        "Petitio tractionis tua aperta est; peer-session-check incipiendum expectatur.",
    ),
    (
        "operation.message.check_failed",
        "peer-session-check cum {conclusion} finivit.",
    ),
    (
        "operation.message.apply_not_started",
        "peer-session-apply pro petitione tractionis tua non incepit.",
    ),
    (
        "operation.message.apply_wait_start",
        "Probationes superatae; peer-session-apply incipiendum expectatur.",
    ),
    (
        "operation.message.apply_failed",
        "peer-session-apply cum {conclusion} finivit.",
    ),
    (
        "operation.message.pull_request_closed",
        "Petitio tractionis tua ante fusionem clausa est.",
    ),
    (
        "operation.message.merge_failed",
        "Fusionem expectans. Conatus fusionis defecit: {error}",
    ),
    ("operation.failure_stage.checks", "Probationes CI"),
    ("operation.failure_stage.preflight", "Praevectio nodi"),
    ("operation.failure_stage.apply", "Applicatio nodi"),
    ("operation.failure_stage.merge", "Fusio"),
    // Routing policy labels
    ("peering_strategy.full_table.label", "Tabula Plena"),
    (
        "peering_strategy.full_table.description",
        "Omnia itinera valida accipere et omnia itinera valida exportare.",
    ),
    ("peering_strategy.transit.label", "Transitus"),
    (
        "peering_strategy.transit.description",
        "Omnia itinera valida accipere et solum praefixiones proprias exactas exportare.",
    ),
    ("peering_strategy.peer.label", "Par"),
    (
        "peering_strategy.peer.description",
        "Solum itinera directa accipere et praefixiones proprias exactas plus itinera inferiora exportare.",
    ),
    ("peering_strategy.downstream.label", "Inferior"),
    (
        "peering_strategy.downstream.description",
        "Solum itinera directa accipere et omnia itinera valida exportare.",
    ),
    // Operation progress labels
    ("operation.progress.branch", "Ramus"),
    ("operation.progress.checks", "Probationes"),
    ("operation.progress.apply", "Applicatio"),
    ("operation.progress.merge", "Fusio"),
    ("operation.progress.done", "Perfectum"),
    // Operation failure labels
    ("operation.failure.stage", "Gradus defectus"),
    ("operation.failure.conclusion", "Eventus"),
    // Prompts (shell-style left labels)
    ("prompt.autopeer", "autopeer"),
    ("prompt.asn", "ASN"),
    ("prompt.auth", "auct"),
    ("prompt.login", "ingressus"),
    ("prompt.key", "clavis"),
    ("prompt.keys", "claves"),
    ("prompt.signature", "subscriptio"),
    ("prompt.signed", "signatum"),
    ("prompt.pubkey", "clav.pub"),
    ("prompt.mntner", "mntner"),
    ("prompt.emails", "electronicae"),
    ("prompt.code", "codex"),
    // Generic loading / errors
    ("status.working", "Laboratur..."),
    (
        "error.ui.node.choose",
        "Unum ex nodis nostris elige antequam pergas",
    ),
    (
        "error.ui.node.choose_inline",
        "Unum ex nodis nostris elige antequam PR aperias",
    ),
    (
        "error.ui.session.missing_config",
        "Sessioni tuae praesenti singula configurationis desunt",
    ),
    (
        "error.ui.operation.wait_inflight",
        "Mutatio in transitu adhuc in hoc nodo currit — exspecta dum finiat.",
    ),
    (
        "error.ui.node.blocked_conflict",
        "Hic nodus conflictu in repositorio nostro impeditur",
    ),
    (
        "error.ui.session.choose_managed_to_retire",
        "Unam ex sessionibus tuis elige antequam eam retrahas",
    ),
    (
        "error.ui.session.choose_managed_to_delete",
        "Sessionem tuam elige antequam eam deleas",
    ),
    (
        "error.ui.auth.authenticate_first",
        "Autenticare antequam pergas",
    ),
    (
        "error.ui.node.choose_default",
        "Unum ex nodis nostris elige antequam pergas",
    ),
    ("error.auth.asn.required", "ASN necessarius est"),
    (
        "error.auth.oidc.provider.missing",
        "Provisor OIDC deest",
    ),
    ("error.request.challenge_id.missing", "challenge_id deest"),
    (
        "error.ui.auth.method.choose_first",
        "Prius modum autenticationis elige.",
    ),
    (
        "error.ui.auth.registry_email.inactive",
        "Autenticatio electronica registri nunc activa non est.",
    ),
    (
        "error.ui.auth.registry_email.choose_maintainer",
        "Prius custodem cum inscriptionibus electronicis registri elige.",
    ),
    (
        "error.ui.auth.registry_email.code.required",
        "Codicem auctoritatis unius usus ex electronica tua inscribe.",
    ),
    (
        "error.auth.method.unavailable",
        "Ille modus autenticationis pro ASN tuo amplius praesto non est.",
    ),
    (
        "error.ui.auth.impersonation.host_auth_first",
        "Unum ex ASN-ibus hospitis configuratis nostris autenticare antequam alium ASN persones.",
    ),
    (
        "error.ui.auth.impersonation.asn.required",
        "ASN quem personare vis inscribe",
    ),
    (
        "error.ui.auth.impersonation.host_session.missing",
        "Nulla sessio ASN hospitis nunc praesto est",
    ),
    (
        "error.ui.auth.impersonation.host_required",
        "Personatio solum praesto est postquam unum ex ASN-ibus hospitis configuratis nostris autenticaveris.",
    ),
    (
        "error.runtime.browser.unavailable",
        "Fenestra navigatri praesto non est",
    ),
    (
        "error.runtime.oidc.redirect_failed",
        "Transmissio ingressus OIDC aperiri non potuit",
    ),
    (
        "error.runtime.config.autopeer_api_url.missing",
        "autopeer_api_url non est configuratus",
    ),
    (
        "error.auth.ssh.empty_or_missing_blocks",
        "Appone integrum tesseram subscriptionis SSH seiunctae ex mandato supra, inclusas lineis BEGIN/END.",
    ),
    (
        "error.auth.ssh.unsigned_challenge",
        "Appone tesseram subscriptionis SSH seiunctae ex mandato supra, non textum provocationis non signatum.",
    ),
    (
        "error.request.body.invalid_json",
        "Corpus petitionis JSON validum esse debet.",
    ),
    (
        "error.auth.asn.unsupported",
        "Illum ambitum ASN nondum sustinemus. Nunc Autopeer solum 424242xxxx sustinet.",
    ),
    (
        "error.auth.asn.not_found",
        "AS{asn} invalidus est quia in registro dn42 non exsistit.",
    ),
    (
        "error.auth.asn.no_supported_auth",
        "AS{asn} in dn42 exsistit, sed auctoritatem custodis quam uti possumus nondum publicat.",
    ),
    (
        "error.auth.registry_email.unavailable",
        "Ingressus electronicus registri in hac deployatione praesto non est.",
    ),
    ("error.auth.challenge.unknown_id", "challenge_id ignotus."),
    (
        "error.auth.challenge.expired",
        "Provocatio autenticationis tuae exspiravit.",
    ),
    (
        "error.auth.challenge.used",
        "Haec provocatio autenticationis iam adhibita est.",
    ),
    (
        "error.auth.session.token.missing",
        "Tessera sessionis bearer deest.",
    ),
    ("error.auth.session.unknown", "Sessio autenticationis ignota."),
    ("error.auth.session.expired", "Sessio autenticationis exspiravit."),
    (
        "error.auth.impersonation.no_maintainers",
        "Hic ASN nullos custodes ad personationem praebitos habet.",
    ),
    (
        "error.auth.ssh.malformed_signature",
        "Data subscriptionis SSH malformata sunt. Reexsequere ssh-keygen -Y sign et integram tesseram subscriptionis seiunctae appone.",
    ),
    (
        "error.auth.ssh.unrecognized_key",
        "Subscriptio SSH tua clave usa est quae in obiectis custodis solutis non adest.",
    ),
    (
        "error.auth.ssh.verification_failed",
        "Verificatio subscriptionis SSH defecit.",
    ),
    (
        "error.auth.pgp.invalid_public_key",
        "Clavis publica PGP invalida est. Clavem tuam publicam loricatam ASCII exporta et integram tesseram appone.",
    ),
    (
        "error.auth.pgp.invalid_signed_message",
        "Nuntius signatus PGP invalidus est. Provocationem clare signa et integram tesseram signatam appone.",
    ),
    (
        "error.auth.pgp.verification_failed",
        "Verificatio subscriptionis PGP defecit. Provocationem cum clave registri congruente resigna et integram tesseram signatam appone.",
    ),
    (
        "error.auth.pgp.unrecognized_key",
        "Vestigium digitale tuum PGP {fingerprint} in obiectis custodis solutis non adest.",
    ),
    (
        "error.auth.pgp.challenge_mismatch",
        "Nuntius tuus signatus PGP provocationi emissae non congruit.",
    ),
    (
        "error.auth.registry_email.state.missing",
        "Status ingressus electronici registri non inventus est vel exspiravit.",
    ),
    (
        "error.auth.registry_email.state.expired",
        "Ingressus electronicus registri exspiravit.",
    ),
    (
        "error.auth.registry_email.state.pending",
        "Ingressus electronicus registri nondum perfectus est.",
    ),
    (
        "error.auth.registry_email.code.invalid",
        "Codex auctoritatis electronicae registri invalidus est.",
    ),
    (
        "error.auth.registry_email.session.missing",
        "Sessio ingressus electronici registri amplius praesto non est.",
    ),
    (
        "error.auth.registry_email.session.expired",
        "Sessio ingressus electronici registri exspiravit.",
    ),
    (
        "error.auth.registry_email.callback.params.missing",
        "Parametri recursus electronici registri desunt.",
    ),
    (
        "error.auth.registry_email.callback.failed",
        "Ingressus electronicus registri defecit; quaeso iterum conare.",
    ),
    (
        "error.auth.registry_email.contacts.missing",
        "AS{asn} nullas inscriptiones electronicas admin-c vel tech-c quas uti possumus in registro exponit.",
    ),
    (
        "error.auth.registry_email.target.missing",
        "{requested} inscriptiones electronicas registri quas pro hoc ASN adhibere possumus non habet.",
    ),
    (
        "error.auth.registry_email.target.required",
        "effective_mnt necessarius est cum autenticatio electronica registri tua plures custodes complectitur.",
    ),
    (
        "error.auth.oidc.callback.provider.missing",
        "Provisor OIDC ex via recursus deest.",
    ),
    (
        "error.auth.oidc.callback.params.missing",
        "Parametri recursus OIDC desunt.",
    ),
    (
        "error.auth.oidc.provider.unknown",
        "Provisor OIDC {provider} ignotus.",
    ),
    (
        "error.auth.oidc.provider.rejected",
        "{error}: {description}",
    ),
    (
        "error.auth.oidc.state.missing",
        "Status ingressus OIDC non inventus est vel exspiravit.",
    ),
    (
        "error.auth.oidc.state.expired",
        "Status ingressus OIDC exspiravit.",
    ),
    (
        "error.auth.oidc.state.pending",
        "Ingressus OIDC nondum perfectus est.",
    ),
    (
        "error.auth.oidc.session.missing",
        "Sessio ingressus OIDC amplius praesto non est.",
    ),
    (
        "error.auth.oidc.session.expired",
        "Sessio ingressus OIDC exspiravit.",
    ),
    (
        "error.auth.oidc.callback.failed",
        "Ingressus OIDC defecit; quaeso iterum conare.",
    ),
    (
        "error.auth.oidc.identity.asn_mismatch",
        "ASN identitatis OIDC {token_asn} ASN petito {requested_asn} non congruit.",
    ),
    (
        "error.auth.session.path_asn_mismatch",
        "ASN viae sessioni autenticatae tuae non congruit.",
    ),
    ("error.request.node.required", "Nodus necessarius est."),
    (
        "error.request.session_payload.required",
        "Sarcina sessionis necessaria est.",
    ),
    (
        "error.auth.impersonation.maintainer.required",
        "effective_mnt necessarius est cum ASN tuus scopus plures custodes habet. Custodes praesto: {available}.",
    ),
    (
        "error.auth.impersonation.maintainer.missing",
        "{requested} in aut-num -> mnt-by pro hoc ASN non adest. Custodes praesto: {available}.",
    ),
    ("error.request.operation.not_found", "Operatio non inventa."),
    ("error.request.operation.not_retryable", "Haec operatio iterari non potest."),
    ("error.request.operation.pr_closed", "Rogatio tractus clausa est et iterari non potest."),
    ("error.request.operation.branch_missing", "Ramus operationis in repositorio deest."),
    ("error.request.route.not_found", "Non inventum."),
    (
        "error.vault.not_configured",
        "Encryptio Vault in hoc servo non configurata est. PSK et encryptio terminationis non praesto sunt.",
    ),
    // Frontend validation
    (
        "validation.tunnel.required",
        "Adde saltem unam inscriptionem cuniculi: IPv4 vel IPv6",
    ),
    (
        "validation.bgp_family.required",
        "Habilita saltem unam familiam BGP",
    ),
    (
        "validation.peer4.required_mp_bgp",
        "Inscriptio IPv4 paris necessaria est pro MP-BGP super transitu IPv4",
    ),
    (
        "validation.peer4.required_ipv4",
        "Inscriptio IPv4 paris necessaria est pro itineribus IPv4",
    ),
    (
        "validation.peer6.required_mp_bgp",
        "Inscriptio IPv6 paris necessaria est pro MP-BGP super transitu IPv6",
    ),
    (
        "validation.peer6.required_ipv6",
        "Inscriptio IPv6 paris necessaria est pro itineribus IPv6",
    ),
    (
        "validation.peer6.required_enh",
        "Inscriptio IPv6 paris necessaria est cum ENH habilitatus est",
    ),
    (
        "validation.extended_next_hop.requires_mp_bgp",
        "Saltus Proximus Extensus MP-BGP requirit",
    ),
    (
        "validation.extended_next_hop.requires_ipv4",
        "Saltus Proximus Extensus itinera IPv4 requirit",
    ),
    (
        "validation.extended_next_hop.requires_ipv6_transport",
        "Saltus Proximus Extensus transitum IPv6 requirit",
    ),
    (
        "validation.ipv4_over_ipv6_transport.requires_peer4_or_enh",
        "IPv4 super transitu IPv6 inscriptionem IPv4 paris vel Saltum Proximum Extensum requirit",
    ),
    (
        "validation.own6.requires_peer6",
        "IPv6 nexus localis proprius inscriptionem IPv6 paris requirit",
    ),
    (
        "validation.own6.requires_link_local_peer6",
        "IPv6 nexus localis proprius solum spectat cum inscriptio IPv6 paris nexus localis est",
    ),
    (
        "validation.own6.must_start_fe80",
        "IPv6 nexus localis proprius incipere debet cum fe80:",
    ),
    (
        "validation.own6.must_differ_from_peer6",
        "IPv6 nexus localis paris differre debet ab IPv6 nexu locali nostro",
    ),
    ("validation.endpoint.required", "terminus necessarius est"),
    (
        "validation.endpoint.no_spaces",
        "Terminus remotus spatia continere non potest",
    ),
    (
        "validation.endpoint.ipv6_format",
        "Termini IPv6 forma [inscr]:portus uti debent",
    ),
    (
        "validation.endpoint.ipv6_invalid",
        "Inscriptio IPv6 termini remoti valida esse debet",
    ),
    (
        "validation.endpoint.host_port_format",
        "Terminus remotus forma nomen:portus vel [ipv6]:portus uti debet",
    ),
    (
        "validation.endpoint.port_required",
        "Terminus remotus portum includere debet",
    ),
    (
        "validation.endpoint.host_required",
        "Nomen termini remoti necessarium est",
    ),
    (
        "validation.endpoint.host_invalid",
        "Nomen termini remoti inscriptio IPv4 valida vel nomen plene qualificatum esse debet",
    ),
    (
        "validation.endpoint.port.invalid",
        "Portus termini remoti numerus validus esse debet",
    ),
    (
        "validation.endpoint.port.range",
        "Portus termini remoti inter 1 et 65535 esse debet",
    ),
    (
        "validation.wg_public_key.required",
        "wg_public_key necessarius est",
    ),
    (
        "validation.wg_public_key.length",
        "Clavis WireGuard paris clavis publica Base64 44 characterum esse debet",
    ),
    (
        "validation.wg_public_key.suffix",
        "Clavis WireGuard paris cum '=' finire debet",
    ),
    (
        "validation.wg_public_key.charset",
        "Clavis WireGuard paris characteres Base64 invalidos continet",
    ),
    (
        "validation.peer4.invalid",
        "Inscriptio IPv4 paris inscriptio IPv4 valida esse debet",
    ),
    (
        "validation.peer4.range",
        "Inscriptio IPv4 paris inscriptio IPv4 dn42 valida esse debet",
    ),
    (
        "validation.peer6.invalid",
        "Inscriptio IPv6 paris inscriptio IPv6 valida esse debet",
    ),
    (
        "validation.peer6.scope",
        "Inscriptio IPv6 paris inscriptio ULA dn42 valida vel nexus localis IPv6 esse debet",
    ),
    (
        "validation.own6.invalid",
        "IPv6 nexus localis proprius inscriptio IPv6 valida esse debet",
    ),
    (
        "validation.own6.scope",
        "IPv6 nexus localis proprius inscriptio IPv6 nexus localis esse debet",
    ),
    (
        "validation.keepalive.invalid",
        "Custodia perpetua numerus validus esse debet",
    ),
    (
        "validation.mtu.invalid",
        "MTU interface numerus validus esse debet",
    ),
    (
        "validation.mtu.range",
        "MTU interface inter 1280 et 1500 esse debet",
    ),
    // Loading messages
    (
        "loading.email_login",
        "Ingressum electronicum tuum perficimus et sessiones tuas oneramus...",
    ),
    (
        "loading.oidc_login",
        "Ingressum OIDC tuum perficimus et sessiones tuas oneramus...",
    ),
    (
        "loading.fetch_sessions",
        "Sessiones tuas praesentes ex repositorio nostro petimus...",
    ),
    (
        "loading.refresh_sessions",
        "Statum sessionis tuae ex repositorio nostro renovamus...",
    ),
    (
        "loading.fetch_methods",
        "Modos autenticationis registri dn42 tui petimus...",
    ),
    (
        "loading.redirect_oidc",
        "Te ad provisorem OIDC tuum transmittimus...",
    ),
    (
        "loading.fetch_challenge",
        "Provocationem recentem registri dn42 pro te petimus...",
    ),
    (
        "loading.send_email",
        "Nexum subscriptionis et codicem unius usus ad inscriptiones electronicas registri tui mittimus...",
    ),
    (
        "loading.check_ssh",
        "Subscriptionem SSH tuam contra registrum dn42 probamus...",
    ),
    (
        "loading.check_pgp",
        "Subscriptionem PGP tuam contra registrum dn42 probamus...",
    ),
    (
        "loading.check_email",
        "Codicem autenticationis electronicae registri tui probamus...",
    ),
    (
        "loading.host_session_prep",
        "Sessionem ASN hospitis tuam paramus...",
    ),
    (
        "loading.authing_asn",
        "ASN contra registrum dn42 autenticamus...",
    ),
    (
        "loading.restore_host",
        "Sessionem ASN hospitis tuam ex repositorio nostro restituimus...",
    ),
    (
        "loading.update_pr",
        "Configurationem peering tuam in repositorio nostro renovamus et petitionem tractionis aperimus...",
    ),
    (
        "loading.create_pr",
        "Configurationem peering tuam in repositorio nostro creamus et petitionem tractionis aperimus...",
    ),
    (
        "loading.retire_pr",
        "Sessionem tuam in repositorio nostro retrahimus et petitionem tractionis aperimus...",
    ),
    (
        "loading.delete_pr",
        "Sessionem e repositorio nostro delemus et rogationem tractus aperimus...",
    ),
    (
        "loading.retry_operation",
        "Operationem defectam iteramus...",
    ),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
