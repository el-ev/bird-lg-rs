#[allow(dead_code)]
pub(super) const TABLE: &[(&str, &str)] = &[
    ("app.title", "dn42 自助对等"),
    ("app.title.footnote", "之于 IRIS-AS 4242421023"),
    ("nav.looking_glass", "镜子"),
    ("nav.language", "语言"),
    // Generic actions
    ("action.back", "返回"),
    ("action.refresh", "刷新"),
    ("action.logout", "登出"),
    ("action.cancel_edit", "取消编辑"),
    ("action.choose_another_node", "选择其他节点"),
    ("action.back_to_nodes", "返回节点列表"),
    ("action.back_to_details", "返回详情"),
    ("action.review_your_update", "审核你的更新"),
    ("action.review_your_change", "审核你的变更"),
    ("action.open_update_pr", "提交更新拉取请求"),
    ("action.open_create_pr", "提交创建拉取请求"),
    ("action.impersonate_this_asn", "冒充此 ASN"),
    ("action.return_to_host_asn", "返回主 ASN"),
    ("action.find_registry_auth", "在注册库中查找认证方式"),
    ("action.verify", "验证"),
    ("action.verify_code", "验证代码"),
    ("action.send_signin_link", "发送登入链接"),
    ("action.resend_signin_link", "重新发送登入链接"),
    ("action.confirm_retirement", "确认停用"),
    ("action.retire_session", "停用会话"),
    ("action.confirm_deletion", "确认删除"),
    ("action.delete_session", "删除会话"),
    ("action.open_pr", "查看拉取请求"),
    ("action.workflow_run", "查看工作流"),
    ("action.retry", "重试"),
    ("action.redeploy", "重新部署"),
    ("action.drop_changes", "放弃更改"),
    ("action.dismiss_operation", "关闭"),
    // Step: LoadingConfig / EnterAsn
    ("step.loading_config.prompt", "加载运行时配置"),
    ("step.loading_config.message", "正在加载运行时配置..."),
    (
        "step.enter_asn.prompt",
        "输入你的 dn42 ASN 以使用注册库 SSH、PGP 或邮件认证。",
    ),
    ("step.enter_asn.placeholder", "424242xxxx"),
    (
        "step.enter_asn.oidc_alt",
        "或使用你的身份提供商登入，让我们自动识别你的 ASN。",
    ),
    ("step.enter_asn.continue_with", "使用 {provider} 继续"),
    // Step: SelectMethod
    (
        "step.select_method.found_for_as",
        "我们在注册库中找到了 AS{asn} 的认证方式",
    ),
    // Backend auth method copy
    ("auth_method.registry_ssh.label", "注册库 SSH 签名"),
    (
        "auth_method.registry_ssh.description",
        "使用来自你的 dn42 维护者（mntner）对象的 SSH 密钥签名我们的质询。",
    ),
    ("auth_method.registry_pgp.label", "注册库 PGP 签名"),
    (
        "auth_method.registry_pgp.description",
        "使用你的注册库 PGP 指纹之一：{fingerprints}",
    ),
    ("auth_method.registry_email.label", "注册库邮件"),
    (
        "auth_method.registry_email.description",
        "选择一个邮件地址，我们会向其发送登入链接和一次性代码。",
    ),
    (
        "auth_method.registry_email.description_single",
        "向 {emails} 发送登入链接和一次性代码。",
    ),
    (
        "auth_method.registry_ssh.session_description",
        "你已使用 SSH 认证作为 {mnt} 登入。",
    ),
    (
        "auth_method.registry_pgp.session_description",
        "你已使用 PGP 认证作为 {mnt} 登入。",
    ),
    (
        "auth_method.registry_email.session_description",
        "你已使用邮件认证作为 {mnt} 登入。",
    ),
    ("auth_method.host_impersonation.label", "主 ASN 冒充"),
    (
        "auth_method.host_impersonation.description",
        "你正通过我们的主 ASN AS{host_asn} 冒充 {mnt}。",
    ),
    (
        "auth_method.oidc.description",
        "使用 {provider} 认证并证明你对此 ASN 的维护者声明之一。",
    ),
    (
        "auth_method.oidc.session_description",
        "你已通过 {provider} 作为 {mnt} 登入。",
    ),
    // Step: VerifyMethod (SSH)
    (
        "verify.ssh.no_fingerprints",
        "我们无法找到用于你的 ASN 的任何 SSH 密钥指纹。",
    ),
    ("verify.ssh.match_one", "匹配你的 SSH 密钥 {fingerprint}"),
    (
        "verify.ssh.match_many",
        "匹配你的 SSH 密钥之一： {fingerprints}",
    ),
    ("verify.ssh.create_signature", "对质询进行签名"),
    (
        "verify.ssh.paste_prompt",
        "运行上方命令，然后粘贴你的独立 SSH 签名块。",
    ),
    ("verify.ssh.placeholder", "-----BEGIN SSH SIGNATURE-----"),
    // Step: VerifyMethod (PGP)
    (
        "verify.pgp.no_fingerprints",
        "我们无法找到用于你的 ASN 的任何 PGP 指纹",
    ),
    ("verify.pgp.use_key", "使用你的密钥 {fingerprint}"),
    (
        "verify.pgp.clearsign_intro",
        "使用匹配的密钥对质询文本进行明文签名，然后导出该公钥并将两者的输出粘贴在下方。",
    ),
    ("verify.pgp.exact_challenge", "质询文本"),
    ("verify.pgp.clearsign_label", "对你的质询进行明文签名"),
    (
        "verify.pgp.signed_paste_prompt",
        "粘贴来自上方命令的完整明文签名质询块",
    ),
    (
        "verify.pgp.signed_placeholder",
        "-----BEGIN PGP SIGNED MESSAGE-----",
    ),
    ("verify.pgp.export_label", "导出你的公钥"),
    (
        "verify.pgp.pubkey_paste_prompt",
        "粘贴来自上方导出命令的 ASCII 保护的公钥",
    ),
    (
        "verify.pgp.pubkey_placeholder",
        "-----BEGIN PGP PUBLIC KEY BLOCK-----",
    ),
    ("verify.pgp.lookup.searching", "正在密钥服务器上查找\u{2026}"),
    ("verify.pgp.lookup.found", "已从密钥服务器获取"),
    ("verify.pgp.lookup.found_from", "已从 {source} 获取"),
    // Step: VerifyMethod (Email)
    (
        "verify.email.intro",
        "发送登入链接和一次性代码至你其中一个维护者的邮箱，然后点击链接或在下方粘贴代码。",
    ),
    (
        "verify.email.no_contacts",
        "我们无法为你的 ASN 找到任何 admin-c 或 tech-c 邮箱联系人。",
    ),
    ("verify.email.auth_as", "认证身份为 {mnt}"),
    ("verify.email.send_to", "发送至 {emails}"),
    (
        "verify.email.sent_to_prefix",
        "我们已发送登入链接和认证代码至 {emails}。",
    ),
    (
        "verify.email.code_prompt",
        "粘贴来自你的邮件的认证代码",
    ),
    ("verify.email.code_placeholder", "12345678"),
    // Step: VerifyMethod (OIDC / Host)
    ("verify.oidc.continue_to", "继续前往 {provider}"),
    (
        "verify.oidc.in_browser",
        "在你的浏览器中前往 {provider}",
    ),
    (
        "verify.oidc.redirect_note",
        "我们将会把你重定向至你的提供商，并在其证明你的 ASN 和维护者声明后带你回到此处。",
    ),
    (
        "verify.host.note",
        "在认证了我们配置的任一主 ASN 后即可使用冒充功能。",
    ),
    (
        "verify.choose_first",
        "请先选择一种认证方式。",
    ),
    ("verify.auth_for_as", "{label} 用于 AS{asn}"),
    // Manage / dashboard headings
    ("dashboard.flow_kicker", "你的对等流程"),
    (
        "dashboard.host_readonly_title",
        "我们的主 ASN 在此保持只读",
    ),
    ("dashboard.update_managed_title", "更新你的会话"),
    (
        "dashboard.create_or_manage_title",
        "创建或管理你的会话",
    ),
    (
        "dashboard.host_readonly_body",
        "我们的主 ASN 仅用于支持其他网络。在创建、更新或停用会话前，请先冒充你想要管理的 ASN。",
    ),
    (
        "dashboard.create_or_manage_body",
        "完成一次认证并选择我们的一个节点。随后你可以创建新会话，或打开现有会话以更新或停用它。",
    ),
    ("dashboard.session_badge_template", "{mnt} 通过 {label} 认证"),
    // Sidebar
    ("sidebar.your_session_kicker", "你的会话"),
    ("sidebar.no_active_session", "无活跃会话"),
    (
        "sidebar.session_authed_template",
        "你已通过 {label} 作为 {mnt} 认证。",
    ),
    ("sidebar.support_kicker", "支持模式"),
    ("sidebar.host_asn_prefix", "主 ASN AS{asn}"),
    (
        "sidebar.host_authed_template",
        "你已通过 {label} 作为 {mnt} 认证。仅在你需要为其他 ASN 打开或修复会话时使用此功能。",
    ),
    ("sidebar.impersonate_asn_label", "冒充的 ASN"),
    ("sidebar.effective_mnt_label", "有效的维护者"),
    ("sidebar.impersonate_asn_placeholder", "424242xxxx"),
    ("sidebar.impersonate_mnt_placeholder", "可选的维护者标识"),
    ("sidebar.current_operation", "当前操作"),
    ("sidebar.support_mode_title", "冒充其他 ASN"),
    (
        "sidebar.support_mode_body",
        "此主 ASN 仅用于帮助其他网络。使用右侧的控件来冒充你想要管理的 ASN。",
    ),
    // Stage 1: Select node
    ("stage1.kicker", "阶段 1"),
    ("stage1.title", "选择我们的一个节点"),
    ("flow.select_node.title", "选择节点"),
    (
        "flow.select_node.description",
        "在填写隧道详情前，选择我们网络中最近的节点。",
    ),
    ("flow.session_details.title", "配置你的会话"),
    (
        "flow.session_details.description",
        "输入你的 WireGuard 和 BGP 参数，然后调整所需的高级选项。",
    ),
    ("flow.review.title", "审核你的变更"),
    (
        "flow.review.description",
        "在我们创建拉取请求前审核你的变更。",
    ),
    (
        "stage1.description",
        "在我们的网络中选择一个节点。空闲节点允许你创建新会话；已有会话将就地打开以供更新。手动会话在保存时将自动纳入自助对等管理。进行中的节点保持只读状态。",
    ),
    (
        "stage1.empty_title",
        "我们未找到你的 ASN 对应的任何已启用自助对等的节点。",
    ),
    (
        "stage1.empty_body",
        "如果此结果有误，请刷新页面或查阅我们的自助对等策略。",
    ),
    ("stage1.state.available", "可用"),
    ("stage1.state.disabled", "已禁用"),
    (
        "stage1.state.note.create",
        "在此节点上创建你的会话。",
    ),
    (
        "stage1.state.note.managed",
        "打开此节点以更新或停用你的会话。",
    ),
    (
        "stage1.state.note.manual",
        "打开此节点以查看当前仓库配置。保存后该会话将自动纳入自助对等管理。",
    ),
    (
        "stage1.state.note.pending",
        "此处你的会话变更已在进行中。",
    ),
    (
        "stage1.state.note.stalled",
        "上次部署失败——点击修改、重新部署或放弃更改。",
    ),
    (
        "stage1.state.note.conflict",
        "我们的仓库在此节点上存在冲突。",
    ),
    (
        "stage1.state.note.disabled",
        "此节点当前不接受自助对等会话。",
    ),
    // Stalled PR banner
    ("stalled.banner.title", "部署失败"),
    (
        "stalled.banner.body",
        "此前的更改有一个未合并的拉取请求部署失败。你可以修改配置后重新提交、重新部署当前 PR，或直接放弃更改。",
    ),
    // Stage 2: Session details
    ("stage2.kicker", "阶段 2"),
    (
        "stage2.title.update_prefix",
        "在 {node} 上更新或停用你的会话",
    ),
    (
        "stage2.title.create_prefix",
        "在 {node} 上配置你的会话",
    ),
    ("stage2.title.create_blank", "配置你的新会话"),
    (
        "stage2.update_intro",
        "你在此节点上已有一个托管会话。你可在下方更新对等详情，或在不再需要时停用该会话。",
    ),
    ("stage2.section.connection", "连接"),
    ("stage2.section.tunnel", "隧道地址"),
    (
        "stage2.section.tunnel.help",
        "使用你侧配置的地址。IPv6 可以是 ULA（如 `fd42:...`）或链路本地（如 `fe80:...`）。",
    ),
    ("stage2.section.families", "路由族"),
    (
        "stage2.section.families.help",
        "选择你的会话应承载的 dn42 路由族。",
    ),
    ("stage2.section.bgp", "BGP 行为"),
    (
        "stage2.section.bgp.help",
        "MP-BGP 通过你所选的 IPv4 或 IPv6 传输层使用单一 BGP 会话承载 IPv4 和/或 IPv6 路由；若禁用此选项，我们将生成独立的 BGP 会话，且扩展下一跳仅适用于通过 IPv6 传输层承载的 IPv4 路由。",
    ),
    ("stage2.section.policy", "路由策略"),
    ("stage2.advanced.summary", "高级选项"),
    ("stage2.field.endpoint", "端点"),
    (
        "stage2.field.endpoint.placeholder",
        "你的路由器的主机名或 IP:端口",
    ),
    ("stage2.field.wg_key", "WireGuard 密钥"),
    (
        "stage2.field.wg_key.placeholder",
        "来自你的路由器的 Base64 公钥",
    ),
    ("stage2.field.peer4", "对端 IPv4 地址"),
    (
        "stage2.field.peer4.placeholder",
        "你的 dn42 IPv4 地址，如 172.21.111.111",
    ),
    ("stage2.field.peer6", "对端 IPv6 地址"),
    (
        "stage2.field.peer6.placeholder",
        "ULA 或链路本地，如 fd42:4242:1023:: 或 fe80::",
    ),
    ("stage2.field.own6_link_local", "我方链路本地 IPv6"),
    (
        "stage2.field.own6_link_local.placeholder",
        "仅在你的对端 IPv6 地址为链路本地时需要",
    ),
    ("stage2.field.own6_node", "我方节点 IPv6"),
    (
        "stage2.field.own6_node.no_inventory",
        "我们的资产清单中未列出此节点的 IPv6 地址。",
    ),
    ("stage2.field.own4_node", "我方节点 IPv4"),
    (
        "stage2.field.own4_node.no_inventory",
        "我们的资产清单中未列出此节点的 IPv4 地址。",
    ),
    ("stage2.field.families", "路由族"),
    ("stage2.field.families.ipv4_label", "IPv4 路由"),
    ("stage2.field.families.ipv6_label", "IPv6 路由"),
    ("stage2.field.bgp_features", "特性"),
    ("stage2.field.bgp.mpbgp_label", "MP-BGP"),
    ("stage2.field.bgp.enh_label", "扩展下一跳"),
    ("stage2.field.bgp.transport", "传输层"),
    ("stage2.field.policy", "策略"),
    ("stage2.field.comment", "备注"),
    (
        "stage2.field.comment.placeholder",
        "关于你的会话的可选备注",
    ),
    ("stage2.field.keepalive", "持久保活"),
    (
        "stage2.field.keepalive.placeholder",
        "你的路由器的可选保活间隔（秒）",
    ),
    ("stage2.field.mtu", "接口 MTU"),
    ("stage2.field.mtu.placeholder", "可选 MTU"),
    ("stage2.field.psk", "预共享密钥"),
    ("stage2.field.psk.placeholder", "可选的 WireGuard PSK"),
    ("stage2.field.psk.placeholder.existing", "已配置 PSK — 留空以保留"),
    ("stage2.field.psk.clear", "清除 PSK"),
    ("stage2.field.psk.generate", "生成 PSK"),
    ("stage2.field.psk.copied", "已复制"),
    ("stage2.field.psk.help", "可选的 WireGuard 预共享密钥，用于增强安全性。密钥将在存储前加密。"),
    ("stage2.field.encrypt_endpoint", "加密"),
    ("stage2.field.encrypt_endpoint.help", "在 Git 仓库中加密你的 Endpoint 地址，使其不以明文形式出现。"),
    // Stage 3: Review
    ("stage3.kicker", "阶段 3"),
    ("stage3.title", "在我们创建拉取请求前审核你的变更"),
    ("stage3.review.our_node", "我方节点"),
    ("stage3.review.not_selected", "未选择"),
    ("stage3.review.endpoint", "端点"),
    ("stage3.review.wg_key", "WireGuard 公钥"),
    ("stage3.review.route_families", "路由族"),
    ("stage3.review.bgp_behavior", "BGP 行为"),
    ("stage3.review.bgp.mpbgp", "MP-BGP"),
    ("stage3.review.bgp.separate", "独立 IPv4/IPv6 会话"),
    ("stage3.review.bgp.enh_suffix", " + 扩展下一跳"),
    ("stage3.review.routing_policy", "路由策略"),
    ("stage3.review.peer4", "对端 IPv4 地址"),
    ("stage3.review.peer6", "对端 IPv6 地址"),
    ("stage3.review.own6", "我方链路本地 IPv6"),
    ("stage3.review.keepalive", "持久保活"),
    ("stage3.review.mtu", "MTU"),
    ("stage3.review.psk", "预共享密钥"),
    ("stage3.review.psk.set", "已配置（已加密）"),
    ("stage3.review.psk.not_set", "未配置"),
    ("stage3.review.psk.unchanged", "已配置（未更改）"),
    ("stage3.review.psk.cleared", "将被移除"),
    ("stage3.review.encrypt_endpoint.enabled", "已加密"),
    ("stage3.review.note", "你的备注"),
    ("stage3.review.our_node_details", "我方节点详情"),
    ("stage3.review.our_endpoint", "端点"),
    ("stage3.review.our_ipv4", "IPv4"),
    ("stage3.review.our_ipv6", "IPv6"),
    ("stage3.review.our_link_local_ipv6", "链路本地 IPv6"),
    ("stage3.review.our_wg_pubkey", "WireGuard 公钥"),
    ("stage3.review.our_node_note", "备注"),
    // Draft / node formatting
    ("draft.families.ipv4_ipv6", "IPv4 + IPv6"),
    ("draft.families.ipv4_only", "仅 IPv4"),
    ("draft.families.ipv6_only", "仅 IPv6"),
    ("draft.families.none", "未选择路由族"),
    ("location.region.europe", "欧洲"),
    ("location.region.north_america_e", "北美洲东部"),
    ("location.region.north_america_c", "北美洲中部"),
    ("location.region.north_america_w", "北美洲西部"),
    ("location.region.central_america", "中美洲"),
    ("location.region.south_america_e", "南美洲东部"),
    ("location.region.south_america_w", "南美洲西部"),
    ("location.region.africa_n", "北非"),
    ("location.region.africa_s", "非洲南部"),
    ("location.region.asia_s", "南亚"),
    ("location.region.asia_se", "东南亚"),
    ("location.region.asia_e", "东亚"),
    ("location.region.asia_n", "北亚"),
    ("location.region.asia_w", "西亚"),
    ("location.region.central_asia", "中亚"),
    ("location.region.pacific_oceania", "太平洋和大洋洲"),
    ("location.region.antarctica", "南极洲"),
    ("location.country.au", "澳大利亚"),
    ("location.country.at", "奥地利"),
    ("location.country.be", "比利时"),
    ("location.country.br", "巴西"),
    ("location.country.bg", "保加利亚"),
    ("location.country.ca", "加拿大"),
    ("location.country.cn", "中国"),
    ("location.country.cz", "捷克"),
    ("location.country.dk", "丹麦"),
    ("location.country.fi", "芬兰"),
    ("location.country.fr", "法国"),
    ("location.country.de", "德国"),
    ("location.country.hk", "香港"),
    ("location.country.hu", "匈牙利"),
    ("location.country.in", "印度"),
    ("location.country.id", "印度尼西亚"),
    ("location.country.ie", "爱尔兰"),
    ("location.country.it", "意大利"),
    ("location.country.jp", "日本"),
    ("location.country.kr", "韩国"),
    ("location.country.lu", "卢森堡"),
    ("location.country.my", "马来西亚"),
    ("location.country.nl", "荷兰"),
    ("location.country.nz", "新西兰"),
    ("location.country.no", "挪威"),
    ("location.country.pl", "波兰"),
    ("location.country.pt", "葡萄牙"),
    ("location.country.ro", "罗马尼亚"),
    ("location.country.ru", "俄罗斯"),
    ("location.country.sg", "新加坡"),
    ("location.country.za", "南非"),
    ("location.country.es", "西班牙"),
    ("location.country.se", "瑞典"),
    ("location.country.ch", "瑞士"),
    ("location.country.tw", "台湾"),
    ("location.country.th", "泰国"),
    ("location.country.tr", "土耳其"),
    ("location.country.ua", "乌克兰"),
    ("location.country.gb", "英国"),
    ("location.country.us", "美国"),
    ("location.country.vn", "越南"),
    ("location.direction.n", "北"),
    ("location.direction.s", "南"),
    ("location.direction.e", "东"),
    ("location.direction.w", "西"),
    ("location.direction.ne", "东北"),
    ("location.direction.nw", "西北"),
    ("location.direction.se", "东南"),
    ("location.direction.sw", "西南"),
    ("node.transport.ipv4", "IPv4"),
    ("node.transport.ipv6", "IPv6"),
    ("node.transport.dual_stack", "双栈"),
    // Session / operation labels
    ("session_state.managed", "托管"),
    ("session_state.manual", "手动"),
    ("session_state.pending_pr", "拉取请求待处理"),
    ("session_state.stalled_pr", "部署失败"),
    ("session_state.conflict", "冲突"),
    ("session.badge.psk", "PSK"),
    ("session.badge.encrypted_endpoint", "Endpoint 已加密"),
    ("operation.kind.create", "创建"),
    ("operation.kind.update", "更新"),
    ("operation.kind.retire", "停用"),
    ("operation.kind.delete", "删除"),
    ("operation.kind.migrate", "迁移"),
    ("operation.state.pending_pull_request", "准备拉取请求"),
    ("operation.state.pending_checks", "等待 CI"),
    ("operation.state.applying", "在节点上应用"),
    ("operation.state.pending_merge", "等待合并"),
    ("operation.state.completed", "已完成"),
    ("operation.state.failed", "失败"),
    ("operation.state.conflict", "冲突"),
    // Backend operation messages
    (
        "operation.message.pending_pull_request",
        "我们正在准备你的拉取请求。",
    ),
    (
        "operation.message.pending_checks",
        "你的拉取请求已打开；正在等待 peer-session-check。",
    ),
    (
        "operation.message.applying",
        "检查已通过；正在将你的会话应用到节点进行验证。",
    ),
    (
        "operation.message.pending_merge",
        "节点上应用成功；正在等待合并。",
    ),
    (
        "operation.message.completed",
        "你的变更已成功应用并合并。",
    ),
    ("operation.message.failed", "你的变更失败。"),
    (
        "operation.message.conflict",
        "由于我们的仓库存在冲突，无法应用你的变更。",
    ),
    (
        "operation.message.wait_node_lock",
        "应用成功；正在等待此节点上的另一项变更完成合并。",
    ),
    (
        "operation.message.no_change",
        "你的会话已与我们的仓库一致，因此未创建拉取请求。",
    ),
    (
        "operation.message.check_not_started",
        "peer-session-check 未能为你的拉取请求启动。",
    ),
    (
        "operation.message.check_wait_start",
        "你的拉取请求已打开；正在等待 peer-session-check 启动。",
    ),
    (
        "operation.message.check_failed",
        "peer-session-check 以 {conclusion} 结束。",
    ),
    (
        "operation.message.apply_not_started",
        "peer-session-apply 未能为你的拉取请求启动。",
    ),
    (
        "operation.message.apply_wait_start",
        "检查已通过；正在等待 peer-session-apply 启动。",
    ),
    (
        "operation.message.apply_failed",
        "peer-session-apply 以 {conclusion} 结束。",
    ),
    (
        "operation.message.pull_request_closed",
        "你的拉取请求在合并前已被关闭。",
    ),
    (
        "operation.message.merge_failed",
        "正在等待合并。合并尝试失败：{error}",
    ),
    (
        "operation.message.dropped",
        "更改已放弃——拉取请求已关闭。",
    ),
    ("operation.failure_stage.checks", "CI 检查"),
    ("operation.failure_stage.preflight", "节点预检"),
    ("operation.failure_stage.apply", "节点应用"),
    ("operation.failure_stage.merge", "合并"),
    // Routing policy labels
    ("peering_strategy.full_table.label", "全表"),
    (
        "peering_strategy.full_table.description",
        "接收所有有效路由并导出所有有效路由。",
    ),
    ("peering_strategy.transit.label", "中转"),
    (
        "peering_strategy.transit.description",
        "接收所有有效路由并仅导出我们自有的前缀。",
    ),
    ("peering_strategy.peer.label", "对等"),
    (
        "peering_strategy.peer.description",
        "仅接收直连路由并导出我们自有的前缀及下游路由。",
    ),
    ("peering_strategy.downstream.label", "下游"),
    (
        "peering_strategy.downstream.description",
        "仅接收直连路由并导出所有有效路由。",
    ),
    // Operation progress labels
    ("operation.progress.branch", "分支"),
    ("operation.progress.checks", "检查"),
    ("operation.progress.apply", "应用"),
    ("operation.progress.merge", "合并"),
    ("operation.progress.done", "完成"),
    // Operation failure labels
    ("operation.failure.stage", "失败阶段"),
    ("operation.failure.conclusion", "结果"),
    // Prompts (shell-style left labels)
    ("prompt.autopeer", "自助对等"),
    ("prompt.asn", "ASN"),
    ("prompt.auth", "认证"),
    ("prompt.login", "登录"),
    ("prompt.key", "密钥"),
    ("prompt.keys", "密钥"),
    ("prompt.signature", "签名"),
    ("prompt.signed", "签名文本"),
    ("prompt.pubkey", "公钥"),
    ("prompt.mntner", "维护者"),
    ("prompt.emails", "邮箱"),
    ("prompt.code", "验证码"),
    // Generic loading / errors
    ("status.working", "处理中..."),
    (
        "error.ui.node.choose",
        "请先选择我们的一个节点再继续",
    ),
    (
        "error.ui.node.choose_inline",
        "请先选择我们的一个节点再创建拉取请求",
    ),
    (
        "error.ui.session.missing_config",
        "你当前的会话缺少配置详情",
    ),
    (
        "error.ui.operation.wait_inflight",
        "此节点上有一项变更仍在进行中——请等待其完成。",
    ),
    (
        "error.ui.node.blocked_conflict",
        "此节点因我们的仓库冲突而被阻止",
    ),
    (
        "error.ui.session.choose_managed_to_retire",
        "请先选择你的一个会话再停用",
    ),
    (
        "error.ui.session.choose_managed_to_delete",
        "请先选择一个会话再删除",
    ),
    (
        "error.ui.auth.authenticate_first",
        "请先完成认证再继续",
    ),
    (
        "error.ui.node.choose_default",
        "请先选择我们的一个节点再继续",
    ),
    ("error.auth.asn.required", "ASN 为必填项"),
    (
        "error.auth.oidc.provider.missing",
        "缺少 OIDC 提供商",
    ),
    ("error.request.challenge_id.missing", "缺少 challenge_id"),
    (
        "error.ui.auth.method.choose_first",
        "请先选择一种认证方式。",
    ),
    (
        "error.ui.auth.registry_email.inactive",
        "注册库邮件认证当前未激活。",
    ),
    (
        "error.ui.auth.registry_email.choose_maintainer",
        "请先选择一个具有注册库邮件联系人的维护者。",
    ),
    (
        "error.ui.auth.registry_email.code.required",
        "请输入来自你邮件的一次性认证代码。",
    ),
    (
        "error.auth.method.unavailable",
        "该认证方式已不再适用于你的 ASN。",
    ),
    (
        "error.ui.auth.impersonation.host_auth_first",
        "在冒充其他 ASN 之前，请先认证我们配置的主 ASN 之一。",
    ),
    (
        "error.ui.auth.impersonation.asn.required",
        "请输入你想要冒充的 ASN",
    ),
    (
        "error.ui.auth.impersonation.host_session.missing",
        "当前无可用的主 ASN 会话",
    ),
    (
        "error.ui.auth.impersonation.host_required",
        "冒充功能仅在认证了我们配置的主 ASN 之一后可用。",
    ),
    (
        "error.runtime.decode_failed",
        "响应解码失败：{detail}",
    ),
    (
        "error.runtime.http_failed",
        "HTTP 请求失败，状态码 {status}",
    ),
    (
        "error.runtime.encode_failed",
        "载荷编码失败：{detail}",
    ),
    (
        "error.runtime.unsupported_method",
        "不支持的 HTTP 方法 {method}",
    ),
    (
        "error.runtime.request_failed",
        "请求失败：{detail}",
    ),
    (
        "error.runtime.config.load_failed",
        "加载 config.json 失败：{detail}",
    ),
    (
        "error.runtime.browser.unavailable",
        "浏览器窗口不可用",
    ),
    (
        "error.runtime.oidc.redirect_failed",
        "无法打开 OIDC 登录重定向",
    ),
    (
        "error.runtime.config.autopeer_api_url.missing",
        "autopeer_api_url 未配置",
    ),
    (
        "error.auth.ssh.empty_or_missing_blocks",
        "请粘贴来自上方命令的完整独立 SSH 签名块，包括 BEGIN/END 行。",
    ),
    (
        "error.auth.ssh.unsigned_challenge",
        "请粘贴来自上方命令的独立 SSH 签名块，而非未签名的质询文本。",
    ),
    (
        "error.request.body.invalid_json",
        "请求体必须为有效的 JSON。",
    ),
    (
        "error.auth.asn.unsupported",
        "我们尚不支持该 ASN 范围。目前自助对等仅支持 424242xxxx。",
    ),
    (
        "error.auth.asn.not_found",
        "AS{asn} 无效，因其不存在于 dn42 注册库中。",
    ),
    (
        "error.auth.asn.no_supported_auth",
        "AS{asn} 存在于 dn42 中，但尚未发布我们可使用的维护者认证信息。",
    ),
    (
        "error.auth.registry_email.unavailable",
        "此部署中不提供注册库邮件登录。",
    ),
    ("error.auth.challenge.unknown_id", "未知的 challenge_id。"),
    (
        "error.auth.challenge.expired",
        "你的认证质询已过期。",
    ),
    (
        "error.auth.challenge.used",
        "此认证质询已被使用。",
    ),
    (
        "error.auth.session.token.missing",
        "缺少 Bearer 会话令牌。",
    ),
    ("error.auth.session.unknown", "未知的认证会话。"),
    ("error.auth.session.expired", "认证会话已过期。"),
    (
        "error.auth.impersonation.no_maintainers",
        "此 ASN 没有可用于冒充的维护者。",
    ),
    (
        "error.auth.ssh.malformed_signature",
        "SSH 签名数据格式错误。请重新运行 ssh-keygen -Y sign 并粘贴完整的独立签名块。",
    ),
    (
        "error.auth.ssh.unrecognized_key",
        "你的 SSH 签名使用的密钥不存在于已解析的维护者对象中。",
    ),
    (
        "error.auth.ssh.verification_failed",
        "SSH 签名验证失败。",
    ),
    (
        "error.auth.pgp.invalid_public_key",
        "PGP 公钥无效。请导出你的 ASCII 保护的公钥并粘贴完整块。",
    ),
    (
        "error.auth.pgp.invalid_signed_message",
        "PGP 签名消息无效。请对质询进行明文签名并粘贴完整签名块。",
    ),
    (
        "error.auth.pgp.verification_failed",
        "PGP 签名验证失败。请使用匹配的注册库密钥重新签名质询并粘贴完整签名块。",
    ),
    (
        "error.auth.pgp.unrecognized_key",
        "你的 PGP 指纹 {fingerprint} 不存在于已解析的维护者对象中。",
    ),
    (
        "error.auth.pgp.challenge_mismatch",
        "你的 PGP 签名消息与签发的质询不匹配。",
    ),
    (
        "error.auth.registry_email.state.missing",
        "注册库邮件登录状态未找到或已过期。",
    ),
    (
        "error.auth.registry_email.state.expired",
        "注册库邮件登录已过期。",
    ),
    (
        "error.auth.registry_email.state.pending",
        "注册库邮件登录尚未完成。",
    ),
    (
        "error.auth.registry_email.code.invalid",
        "注册库邮件认证代码无效。",
    ),
    (
        "error.auth.registry_email.session.missing",
        "注册库邮件登录会话已不可用。",
    ),
    (
        "error.auth.registry_email.session.expired",
        "注册库邮件登录会话已过期。",
    ),
    (
        "error.auth.registry_email.callback.params.missing",
        "缺少注册库邮件回调参数。",
    ),
    (
        "error.auth.registry_email.callback.failed",
        "注册库邮件登录失败；请重试。",
    ),
    (
        "error.auth.registry_email.contacts.missing",
        "AS{asn} 未在注册库中公开任何我们可使用的 admin-c 或 tech-c 邮箱地址。",
    ),
    (
        "error.auth.registry_email.target.missing",
        "{requested} 没有我们可用于此 ASN 的注册库邮件联系人。",
    ),
    (
        "error.auth.registry_email.target.required",
        "当你的注册库邮件认证涉及多个维护者时，effective_mnt 为必填项。",
    ),
    (
        "error.auth.oidc.callback.provider.missing",
        "回调路径中缺少 OIDC 提供商。",
    ),
    (
        "error.auth.oidc.callback.params.missing",
        "缺少 OIDC 回调参数。",
    ),
    (
        "error.auth.oidc.provider.unknown",
        "未知的 OIDC 提供商 {provider}。",
    ),
    (
        "error.auth.oidc.provider.rejected",
        "{error}：{description}",
    ),
    (
        "error.auth.oidc.state.missing",
        "OIDC 登录状态未找到或已过期。",
    ),
    (
        "error.auth.oidc.state.expired",
        "OIDC 登录状态已过期。",
    ),
    (
        "error.auth.oidc.state.pending",
        "OIDC 登录尚未完成。",
    ),
    (
        "error.auth.oidc.session.missing",
        "OIDC 登录会话已不可用。",
    ),
    (
        "error.auth.oidc.session.expired",
        "OIDC 登录会话已过期。",
    ),
    (
        "error.auth.oidc.callback.failed",
        "OIDC 登录失败；请重试。",
    ),
    (
        "error.auth.oidc.identity.asn_mismatch",
        "OIDC 身份 ASN {token_asn} 与请求的 ASN {requested_asn} 不匹配。",
    ),
    (
        "error.auth.session.path_asn_mismatch",
        "路径 ASN 与你的已认证会话不匹配。",
    ),
    ("error.request.node.required", "节点为必填项。"),
    (
        "error.request.session_payload.required",
        "会话载荷为必填项。",
    ),
    (
        "error.auth.impersonation.maintainer.required",
        "当目标 ASN 有多个维护者时，effective_mnt 为必填项。可用维护者：{available}。",
    ),
    (
        "error.auth.impersonation.maintainer.missing",
        "{requested} 不存在于此 ASN 的 aut-num -> mnt-by 中。可用维护者：{available}。",
    ),
    ("error.request.operation.not_found", "操作未找到。"),
    ("error.request.operation.not_retryable", "此操作无法重试。"),
    ("error.request.operation.not_droppable", "此操作无法放弃。"),
    ("error.request.operation.pr_closed", "拉取请求已关闭，无法重试。"),
    ("error.request.operation.branch_missing", "操作分支在仓库中缺失。"),
    ("error.request.route.not_found", "未找到。"),
    (
        "error.vault.not_configured",
        "此服务器未配置 Vault 加密。PSK 和 Endpoint 加密功能不可用。",
    ),
    // Backend error messages
    (
        "error.repo.inventory.missing",
        "网络仓库缺少 inventory.yaml",
    ),
    (
        "error.repo.peer_file.missing",
        "网络仓库缺少 {path}",
    ),
    (
        "error.node.not_eligible",
        "节点 {node} 不符合 Autopeer 条件",
    ),
    (
        "error.node.not_accepting_changes",
        "{node} 当前不接受 Autopeer 变更",
    ),
    (
        "error.session.duplicate_on_node",
        "AS{asn} 在 {node} 上已有会话或待处理操作",
    ),
    (
        "error.auth.asn.no_registry_auth.oidc_hint",
        "AS{asn} 未公开支持的注册库 SSH、PGP 或邮件认证方式。请改用已配置的 OIDC 登录选项。",
    ),
    (
        "error.auth.impersonation.host_asn.cannot_mutate",
        "AS{asn} 是我们的主 ASN 会话之一；请先冒充你想要管理的 ASN，再进行开启或修改操作",
    ),
    (
        "error.auth.impersonation.asn.not_host",
        "AS{asn} 未被配置为可冒充的主 ASN",
    ),
    (
        "error.auth.registry_email.already_completed",
        "注册库邮件登录已完成；请通过已发送的邮件登录链接完成操作。",
    ),
    (
        "error.request.session.mp_bgp_transport.invalid",
        "session.mp_bgp_transport 必须为 ipv4 或 ipv6 之一",
    ),
    (
        "error.request.session_payload.invalid",
        "会话载荷无效",
    ),
    (
        "error.peer.duplicate",
        "对端文件中存在重复的 ASN AS{asn}",
    ),
    (
        "error.peer.create.session_required",
        "创建操作需要会话载荷",
    ),
    (
        "error.peer.managed.already_exists",
        "托管对端 AS{asn} 已存在于此节点",
    ),
    (
        "error.peer.not_found",
        "对端 AS{asn} 在此节点上不存在",
    ),
    (
        "error.peer.already_managed",
        "对端 AS{asn} 已由 Autopeer 管理",
    ),
    (
        "error.peer.update.session_required",
        "更新操作需要会话载荷",
    ),
    (
        "error.peer.manual.cannot_modify",
        "手动对端 AS{asn} 无法由 Autopeer 修改",
    ),
    (
        "error.data.yaml_root.invalid",
        "YAML 根节点必须为映射",
    ),
    (
        "error.data.peer_entry.invalid",
        "对端条目必须为映射",
    ),
    (
        "error.data.peer_entry.missing_bgp",
        "对端条目缺少 BGP 映射",
    ),
    (
        "error.data.peer_entry.missing_asn",
        "对端条目缺少有效的 bgp.asn",
    ),
    (
        "error.data.peer.missing_wg",
        "活跃对端 AS{asn} 缺少 WireGuard 映射",
    ),
    (
        "error.data.peer_file.missing_peers",
        "对端文件必须包含顶层 peers 列表",
    ),
    (
        "error.data.inventory.missing_all",
        "inventory.yaml 缺少顶层 all 键",
    ),
    (
        "error.data.inventory.missing_children",
        "inventory.yaml 缺少 all.children",
    ),
    (
        "error.data.inventory.missing_hosts",
        "inventory.yaml 必须定义 nodes.hosts 和 dn42.hosts",
    ),
    // Frontend validation
    (
        "validation.tunnel.required",
        "至少添加一个隧道地址：IPv4 或 IPv6",
    ),
    (
        "validation.bgp_family.required",
        "至少启用一个 BGP 路由族",
    ),
    (
        "validation.peer4.required_mp_bgp",
        "MP-BGP 基于 IPv4 传输层时需要对端 IPv4 地址",
    ),
    (
        "validation.peer4.required_ipv4",
        "IPv4 路由需要对端 IPv4 地址",
    ),
    (
        "validation.peer6.required_mp_bgp",
        "需要对端 IPv6 地址，或在高级选项中切换为 IPv4 传输",
    ),
    (
        "validation.peer6.required_ipv6",
        "IPv6 路由需要对端 IPv6 地址",
    ),
    (
        "validation.peer6.required_enh",
        "启用扩展下一跳时需要对端 IPv6 地址",
    ),
    (
        "validation.extended_next_hop.requires_mp_bgp",
        "扩展下一跳需要启用 MP-BGP",
    ),
    (
        "validation.extended_next_hop.requires_ipv4",
        "扩展下一跳需要 IPv4 路由",
    ),
    (
        "validation.extended_next_hop.requires_ipv6_transport",
        "扩展下一跳需要 IPv6 传输层",
    ),
    (
        "validation.ipv4_over_ipv6_transport.requires_peer4_or_enh",
        "通过 IPv6 传输层承载 IPv4 需要对端 IPv4 地址或扩展下一跳",
    ),
    (
        "validation.own6.requires_peer6",
        "本地链路本地 IPv6 需要对端 IPv6 地址",
    ),
    (
        "validation.own6.requires_link_local_peer6",
        "本地链路本地 IPv6 仅在对端 IPv6 地址为链路本地时适用",
    ),
    (
        "validation.own6.must_start_fe80",
        "本地链路本地 IPv6 必须以 fe80: 开头",
    ),
    (
        "validation.own6.must_differ_from_peer6",
        "对端链路本地 IPv6 必须与我方链路本地 IPv6 不同",
    ),
    (
        "validation.endpoint.no_spaces",
        "远程端点不能包含空格",
    ),
    (
        "validation.endpoint.ipv6_format",
        "IPv6 端点必须使用 [addr]:port 格式",
    ),
    (
        "validation.endpoint.ipv6_invalid",
        "远程端点 IPv6 地址必须为有效的 IPv6 地址",
    ),
    (
        "validation.endpoint.host_port_format",
        "远程端点必须使用 host:port 或 [ipv6]:port 格式",
    ),
    (
        "validation.endpoint.port_required",
        "远程端点必须包含端口",
    ),
    (
        "validation.endpoint.host_required",
        "远程端点主机为必填项",
    ),
    (
        "validation.endpoint.host_invalid",
        "远程端点主机必须为 IPv4 地址或完整域名",
    ),
    (
        "validation.endpoint.port.invalid",
        "远程端点端口必须为有效数字",
    ),
    (
        "validation.endpoint.port.range",
        "远程端点端口必须在 1 到 65535 之间",
    ),
    (
        "validation.wg_public_key.required",
        "wg_public_key 为必填项",
    ),
    (
        "validation.wg_public_key.length",
        "对端 WireGuard 密钥必须为 44 字符的 Base64 公钥",
    ),
(
        "validation.wg_public_key.charset",
        "对端 WireGuard 密钥包含无效的 Base64 字符",
    ),
    (
        "validation.peer4.invalid",
        "对端 IPv4 地址必须为有效的 IPv4 地址",
    ),
    (
        "validation.peer4.range",
        "对端 IPv4 地址必须为有效的 dn42 IPv4 地址",
    ),
    (
        "validation.peer6.invalid",
        "对端 IPv6 地址必须为有效的 IPv6 地址",
    ),
    (
        "validation.peer6.scope",
        "对端 IPv6 地址必须为有效的 dn42 ULA 或链路本地 IPv6 地址",
    ),
    (
        "validation.own6.invalid",
        "本地链路本地 IPv6 必须为有效的 IPv6 地址",
    ),
    (
        "validation.own6.scope",
        "本地链路本地 IPv6 必须为链路本地 IPv6 地址",
    ),
    (
        "validation.keepalive.invalid",
        "持久保活必须为有效数字",
    ),
    (
        "validation.mtu.invalid",
        "接口 MTU 必须为有效数字",
    ),
    (
        "validation.mtu.range",
        "接口 MTU 必须在 1280 到 1500 之间",
    ),
    (
        "validation.psk.length",
        "预共享密钥必须为 44 字符的 Base64 密钥",
    ),
(
        "validation.psk.charset",
        "预共享密钥包含无效的 Base64 字符",
    ),
    // Backend-only validation
    (
        "validation.mp_bgp_transport.invalid",
        "MP-BGP 传输层必须为 ipv4 或 ipv6 之一",
    ),
    (
        "validation.peering_strategy.invalid",
        "互联策略必须为 standard 或 aggressive",
    ),
    (
        "validation.port.range",
        "端口必须在 1 到 65535 之间",
    ),
    (
        "validation.endpoint.required",
        "远程端点为必填项",
    ),
    (
        "validation.endpoint.node_ipv6_only",
        "{node} 仅支持 IPv6；请使用域名或 IPv6 端点",
    ),
    (
        "validation.endpoint.node_ipv4_only",
        "{node} 仅支持 IPv4；请使用域名或 IPv4 端点",
    ),
    // Loading messages
    (
        "loading.email_login",
        "正在完成你的邮件登录并加载会话...",
    ),
    (
        "loading.oidc_login",
        "正在完成你的 OIDC 登录并加载会话...",
    ),
    (
        "loading.fetch_sessions",
        "正在从我们的仓库获取你的当前会话...",
    ),
    (
        "loading.refresh_sessions",
        "正在从我们的仓库刷新你的会话状态...",
    ),
    (
        "loading.fetch_methods",
        "正在获取你的 dn42 注册库认证方式...",
    ),
    (
        "loading.redirect_oidc",
        "正在将你重定向至你的 OIDC 提供商...",
    ),
    (
        "loading.fetch_challenge",
        "正在为你获取新的 dn42 注册库质询...",
    ),
    (
        "loading.send_email",
        "正在向你的注册库邮件联系人发送登入链接和一次性代码...",
    ),
    (
        "loading.check_ssh",
        "正在验证你的 SSH 签名...",
    ),
    (
        "loading.check_pgp",
        "正在验证你的 PGP 签名...",
    ),
    (
        "loading.check_email",
        "正在验证你的注册库邮件认证代码...",
    ),
    (
        "loading.host_session_prep",
        "正在准备你的主 ASN 会话...",
    ),
    (
        "loading.authing_asn",
        "正在根据 dn42 注册库认证该 ASN...",
    ),
    (
        "loading.restore_host",
        "正在从我们的仓库恢复你的主 ASN 会话...",
    ),
    (
        "loading.update_pr",
        "正在更新我们仓库中你的对等配置并创建拉取请求...",
    ),
    (
        "loading.create_pr",
        "正在我们的仓库中创建你的对等配置并创建拉取请求...",
    ),
    (
        "loading.retire_pr",
        "正在我们的仓库中停用你的会话并创建拉取请求...",
    ),
    (
        "operation.message.workflow_failed",
        "工作流在 {stage} 阶段失败（{conclusion}）",
    ),
    (
        "operation.message.workflow_failed.step",
        "工作流在 {stage} 阶段的步骤「{step}」失败（{conclusion}）",
    ),
    (
        "operation.message.workflow_failed.full",
        "工作流在 {stage} 阶段的步骤「{step}」失败：{annotation}（{conclusion}）",
    ),
    (
        "loading.delete_pr",
        "正在从仓库中删除会话并创建拉取请求……",
    ),
    (
        "loading.retry_operation",
        "正在重试失败的操作……",
    ),
    (
        "loading.drop_operation",
        "正在放弃更改并关闭拉取请求……",
    ),
    // Worker validation / infrastructure errors
    ("error.field.required", "{field} 不能为空。"),
    ("error.field.must_be_string", "{field} 必须为字符串。"),
    ("error.field.must_be_boolean", "{field} 必须为布尔值。"),
    ("error.field.must_be_integer", "{field} 必须为整数。"),
    ("error.field.must_be_object", "{field} 必须为对象。"),
    ("error.asn.format", "ASN 必须形如 424242xxxx。"),
    (
        "error.node.lock.unreadable",
        "无法读取节点 {node} 的锁。",
    ),
    (
        "error.email.send_failed",
        "登录邮件发送失败：{detail}",
    ),
    (
        "error.github.api_failed",
        "GitHub 请求 {path} 失败（HTTP {status}）。",
    ),
    (
        "error.github.file_read_failed",
        "无法从 GitHub 读取 {path}（HTTP {status}）。",
    ),
    (
        "error.registry.request_failed",
        "Registry 请求 {path} 失败（HTTP {status}）。",
    ),
    (
        "error.registry.invalid_payload",
        "Registry 对 {path} 返回了意外的响应内容。",
    ),
    (
        "error.registry.lookup_failed",
        "查询 DN42 Registry AS{asn} 时失败。",
    ),
    (
        "error.oidc.claim.asn_missing",
        "OIDC 身份缺少所需的 ASN claim（{claim}）。",
    ),
    (
        "error.oidc.claim.maintainer_missing",
        "OIDC 身份缺少所需的维护者 claim（{claim}）。",
    ),
    (
        "error.oidc.maintainer.not_in_mnt_by",
        "{provider} 声明了 {candidates}，但其不在 aut-num mnt-by 中。",
    ),
    (
        "error.oidc.discovery.failed",
        "{provider} 的 OIDC 发现失败（HTTP {status}）。",
    ),
    (
        "error.oidc.discovery.invalid_json",
        "{provider} 的 OIDC 发现返回了无效 JSON。",
    ),
    (
        "error.oidc.discovery.missing_field",
        "{provider} 的 OIDC 发现缺少 {field}。",
    ),
    (
        "error.oidc.client_secret.missing",
        "{provider} 缺少用于 {method} 的 client_secret_env。",
    ),
    (
        "error.oidc.token.invalid_json",
        "{provider} 的 token 端点返回了无效 JSON。",
    ),
    (
        "error.oidc.token.rejected",
        "{provider} 拒绝了登录回调：{description}",
    ),
    (
        "error.oidc.userinfo.failed",
        "{provider} userinfo 请求失败（HTTP {status}）。",
    ),
    (
        "error.oidc.userinfo.invalid_json",
        "{provider} userinfo 端点返回了无效 JSON。",
    ),
    (
        "error.oidc.id_token.missing",
        "{provider} 未返回 ID token。",
    ),
    (
        "error.oidc.id_token.invalid",
        "{provider} ID token 验证失败：{detail}",
    ),
    (
        "error.oidc.id_token.invalid_nonce",
        "{provider} 返回的登录 token nonce 无效。",
    ),
    (
        "error.oidc.asn.mismatch",
        "OIDC 身份的 ASN {token_asn} 与请求的 ASN {requested_asn} 不匹配。",
    ),
];

pub(super) fn lookup(key: &str) -> Option<&'static str> {
    TABLE
        .iter()
        .find_map(|(k, v)| if *k == key { Some(*v) } else { None })
}
