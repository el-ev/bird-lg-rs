use std::collections::BTreeSet;

use common::auto_peer::{
    AuthMethod, AuthMethodKind, AuthSessionResponse, CreateSessionRequest, NodeView,
    OperationStatus, RegistryEmailTarget, SessionListResponse, SessionView, UpdateSessionRequest,
};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;
use web_sys::UrlSearchParams;
use yew::prelude::*;

use crate::{
    service,
    store::{
        AutoPeerStep, PeerConfigStage, PersistedSessions, SessionDraft, load_persisted_sessions,
        save_persisted_sessions,
    },
};

const MISSING_AUTOPEER_URL_ERROR: &str = "error.autopeer_url_missing";

#[derive(Clone)]
struct SessionHandles {
    asn: UseStateHandle<String>,
    nodes: UseStateHandle<Vec<NodeView>>,
    sessions: UseStateHandle<Vec<SessionView>>,
    draft: UseStateHandle<SessionDraft>,
    editing_node: UseStateHandle<Option<String>>,
    config_stage: UseStateHandle<PeerConfigStage>,
}

#[derive(Clone)]
struct AuthHandles {
    auth_session: UseStateHandle<Option<AuthSessionResponse>>,
    host_session: UseStateHandle<Option<AuthSessionResponse>>,
    step: UseStateHandle<AutoPeerStep>,
}

#[derive(Clone)]
struct AuthFlowHandles {
    challenge_id: UseStateHandle<Option<String>>,
    challenge_text: UseStateHandle<Option<String>>,
    methods: UseStateHandle<Vec<AuthMethod>>,
    selected_method: UseStateHandle<Option<AuthMethod>>,
    selected_pgp_key: UseStateHandle<String>,
    ssh_signature: UseStateHandle<String>,
    pgp_public_key: UseStateHandle<String>,
    pgp_signed_message: UseStateHandle<String>,
    selected_email_maintainer: UseStateHandle<String>,
    registry_email_code: UseStateHandle<String>,
    registry_email_sent_to: UseStateHandle<Vec<String>>,
}

pub(crate) fn validate_ssh_signature_input(signature: &str) -> Result<(), &'static str> {
    let trimmed = signature.trim();
    if trimmed.is_empty() {
        return Err("error.ssh.empty_or_missing_blocks");
    }
    if !trimmed.contains("-----BEGIN SSH SIGNATURE-----")
        || !trimmed.contains("-----END SSH SIGNATURE-----")
    {
        if trimmed.contains("dn42-autopeer challenge") {
            return Err("error.ssh.unsigned_challenge");
        }
        return Err("error.ssh.empty_or_missing_blocks");
    }
    Ok(())
}

pub(crate) fn default_pgp_key(method: &AuthMethod) -> String {
    method.pgp_fingerprints.first().cloned().unwrap_or_default()
}

pub(crate) fn default_registry_email_target(method: &AuthMethod) -> String {
    method
        .email_targets
        .first()
        .map(|target| target.maintainer.clone())
        .unwrap_or_default()
}

pub(crate) fn selected_registry_email_target<'a>(
    method: &'a AuthMethod,
    selected_maintainer: &str,
) -> Option<&'a RegistryEmailTarget> {
    let selected = selected_maintainer.trim();
    if !selected.is_empty() {
        method
            .email_targets
            .iter()
            .find(|target| target.maintainer == selected)
    } else {
        method.email_targets.first()
    }
}

pub(crate) fn filter_supported_methods(
    methods: Vec<AuthMethod>,
    oidc_enabled: bool,
) -> Vec<AuthMethod> {
    methods
        .into_iter()
        .filter(|method| oidc_enabled || method.kind != AuthMethodKind::Oidc)
        .collect()
}

pub(crate) fn configured_href(configured: Option<&str>, fallback: &str) -> String {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

pub(crate) fn sync_create_draft(
    asn: &str,
    nodes: &[NodeView],
    sessions: &[SessionView],
    current: &SessionDraft,
) -> SessionDraft {
    let mut draft = if current.endpoint.is_empty()
        && current.wg_public_key.is_empty()
        && current.peer6.is_empty()
        && current.node.is_empty()
    {
        SessionDraft::default_for_asn(asn)
    } else {
        current.clone()
    };

    let current_is_selectable = !draft.node.is_empty()
        && nodes.iter().any(|node| node.name == draft.node)
        && sessions.iter().all(|session| session.node != draft.node);

    if !current_is_selectable {
        draft.node.clear();
    }

    draft
}

fn reset_session_selection(handles: &SessionHandles) {
    handles.editing_node.set(None);
    handles.config_stage.set(PeerConfigStage::SelectNode);
}

fn reset_loaded_sessions(handles: &SessionHandles, asn: &str) {
    handles.nodes.set(Vec::new());
    handles.sessions.set(Vec::new());
    handles.draft.set(SessionDraft::default_for_asn(asn));
    reset_session_selection(handles);
}

fn clear_session_state(handles: &SessionHandles) {
    handles.asn.set(String::new());
    handles.nodes.set(Vec::new());
    handles.sessions.set(Vec::new());
    handles.draft.set(SessionDraft::default());
    reset_session_selection(handles);
}

fn clear_auth_inputs(handles: &AuthFlowHandles) {
    handles.ssh_signature.set(String::new());
    handles.pgp_public_key.set(String::new());
    handles.pgp_signed_message.set(String::new());
    handles.registry_email_code.set(String::new());
    handles.registry_email_sent_to.set(Vec::new());
}

fn clear_selected_auth_method(handles: &AuthFlowHandles) {
    handles.selected_method.set(None);
    handles.selected_pgp_key.set(String::new());
    handles.selected_email_maintainer.set(String::new());
    clear_auth_inputs(handles);
}

fn set_auth_challenge(
    handles: &AuthFlowHandles,
    challenge_id: String,
    challenge_text: String,
    methods: Vec<AuthMethod>,
) {
    handles.challenge_id.set(Some(challenge_id));
    handles.challenge_text.set(Some(challenge_text));
    handles.methods.set(methods);
    clear_selected_auth_method(handles);
}

fn clear_auth_challenge(handles: &AuthFlowHandles) {
    handles.challenge_id.set(None);
    handles.challenge_text.set(None);
    handles.methods.set(Vec::new());
    clear_selected_auth_method(handles);
}

fn set_selected_auth_method(handles: &AuthFlowHandles, method: AuthMethod) {
    handles.selected_pgp_key.set(default_pgp_key(&method));
    handles
        .selected_email_maintainer
        .set(default_registry_email_target(&method));
    handles.selected_method.set(Some(method));
    clear_auth_inputs(handles);
}

fn matching_auth_method(
    available_methods: &[AuthMethod],
    method: &AuthMethod,
) -> Option<AuthMethod> {
    available_methods
        .iter()
        .find(|candidate| candidate.kind == method.kind && candidate.provider == method.provider)
        .cloned()
}

fn clear_impersonation_inputs(
    impersonate_asn: &UseStateHandle<String>,
    impersonate_mnt: &UseStateHandle<String>,
) {
    impersonate_asn.set(String::new());
    impersonate_mnt.set(String::new());
}

fn apply_session_list(response: SessionListResponse, handles: &SessionHandles) {
    handles.asn.set(response.asn.clone());
    handles.nodes.set(response.nodes.clone());
    handles.sessions.set(response.sessions.clone());

    if handles.editing_node.is_none() {
        let next = sync_create_draft(
            &response.asn,
            &response.nodes,
            &response.sessions,
            &handles.draft,
        );
        handles.draft.set(next);
    }
}

fn is_stale_session_error(message: &str) -> bool {
    matches!(
        message,
        "unknown auth session" | "auth session has expired" | "missing bearer session token"
    )
}

fn require_api_base(
    api_base: &UseStateHandle<Option<String>>,
    error: &UseStateHandle<Option<String>>,
) -> Option<String> {
    let value = api_base
        .as_ref()
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if value.is_none() {
        error.set(Some(MISSING_AUTOPEER_URL_ERROR.to_string()));
    }

    value
}

fn set_authenticated_session(
    auth_session: &UseStateHandle<Option<AuthSessionResponse>>,
    host_session: &UseStateHandle<Option<AuthSessionResponse>>,
    session: AuthSessionResponse,
) {
    if session.can_impersonate {
        host_session.set(Some(session.clone()));
    }
    auth_session.set(Some(session));
}

fn start_loading(
    loading: &UseStateHandle<bool>,
    loading_message: &UseStateHandle<Option<String>>,
    message: &str,
) {
    loading.set(true);
    loading_message.set(Some(message.to_string()));
}

fn clear_loading(loading: &UseStateHandle<bool>, loading_message: &UseStateHandle<Option<String>>) {
    loading.set(false);
    loading_message.set(None);
}

fn hash_param(name: &str) -> Option<String> {
    web_sys::window().and_then(|window| {
        let hash = window.location().hash().ok()?;
        let query = hash.strip_prefix('#').unwrap_or(&hash);
        let params = UrlSearchParams::new_with_str(query).ok()?;
        params.get(name)
    })
}

fn clear_location_hash() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash("");
    }
}

fn redirect_to(url: &str) -> Result<(), String> {
    let Some(window) = web_sys::window() else {
        return Err("error.browser_unavailable".to_string());
    };
    window
        .location()
        .set_href(url)
        .map_err(|_| "error.oidc_redirect_failed".to_string())
}

fn session_for_node<'a>(node_name: &str, sessions: &'a [SessionView]) -> Option<&'a SessionView> {
    sessions.iter().find(|session| session.node == node_name)
}

fn selected_session_node_name(editing_node: Option<&str>, draft: &SessionDraft) -> Option<String> {
    editing_node
        .map(str::to_string)
        .or_else(|| (!draft.node.trim().is_empty()).then(|| draft.node.trim().to_string()))
}

async fn activate_authenticated_session(
    api_base: &str,
    session: AuthSessionResponse,
    session_handles: &SessionHandles,
    auth_handles: &AuthHandles,
    error: &UseStateHandle<Option<String>>,
) {
    let session_asn = session.asn.clone();
    match service::list_sessions(api_base, &session.session_token).await {
        Ok(response) => {
            set_authenticated_session(
                &auth_handles.auth_session,
                &auth_handles.host_session,
                session,
            );
            apply_session_list(response, session_handles);
            let next_draft = sync_create_draft(
                &session_asn,
                &session_handles.nodes,
                &session_handles.sessions,
                &session_handles.draft,
            );
            session_handles.draft.set(next_draft);
            reset_session_selection(session_handles);
            error.set(None);
            auth_handles.step.set(AutoPeerStep::ManageSessions);
        }
        Err(message) => {
            set_authenticated_session(
                &auth_handles.auth_session,
                &auth_handles.host_session,
                session,
            );
            session_handles.asn.set(session_asn.clone());
            reset_loaded_sessions(session_handles, &session_asn);
            error.set(Some(message));
            auth_handles.step.set(AutoPeerStep::ManageSessions);
        }
    }
}

async fn finish_redirected_auth_session(
    api_url: &str,
    result: Result<AuthSessionResponse, String>,
    session_handles: &SessionHandles,
    auth_handles: &AuthHandles,
    error: &UseStateHandle<Option<String>>,
    loading: &UseStateHandle<bool>,
    loading_message: &UseStateHandle<Option<String>>,
) {
    clear_location_hash();
    match result {
        Ok(session) => {
            activate_authenticated_session(api_url, session, session_handles, auth_handles, error)
                .await;
        }
        Err(message) => {
            error.set(Some(message));
            auth_handles.step.set(AutoPeerStep::EnterAsn);
        }
    }
    clear_loading(loading, loading_message);
}

async fn restore_persisted_state(
    api_url: &str,
    persisted: PersistedSessions,
    session_handles: &SessionHandles,
    auth_handles: &AuthHandles,
    error: &UseStateHandle<Option<String>>,
) {
    let mut valid_host_session = None::<AuthSessionResponse>;
    let mut valid_host_response = None::<SessionListResponse>;

    if let Some(host) = persisted.host_session.clone() {
        match service::list_sessions(api_url, &host.session_token).await {
            Ok(response) => {
                valid_host_response = Some(response);
                valid_host_session = Some(host);
            }
            Err(message) if is_stale_session_error(&message) => {}
            Err(message) => {
                error.set(Some(message));
                auth_handles.step.set(AutoPeerStep::EnterAsn);
                return;
            }
        }
    }

    if let Some(active) = persisted.auth_session.clone() {
        match service::list_sessions(api_url, &active.session_token).await {
            Ok(response) => {
                auth_handles.auth_session.set(Some(active.clone()));
                auth_handles.host_session.set(valid_host_session.clone());
                apply_session_list(response, session_handles);
                save_persisted_sessions(&PersistedSessions {
                    auth_session: Some(active),
                    host_session: valid_host_session,
                });
                reset_session_selection(session_handles);
                error.set(None);
                auth_handles.step.set(AutoPeerStep::ManageSessions);
                return;
            }
            Err(message) if is_stale_session_error(&message) => {}
            Err(message) => {
                error.set(Some(message));
                auth_handles.step.set(AutoPeerStep::EnterAsn);
                return;
            }
        }
    }

    if let (Some(host), Some(response)) = (valid_host_session.clone(), valid_host_response) {
        auth_handles.auth_session.set(Some(host.clone()));
        auth_handles.host_session.set(Some(host.clone()));
        apply_session_list(response, session_handles);
        save_persisted_sessions(&PersistedSessions {
            auth_session: Some(host.clone()),
            host_session: Some(host),
        });
        reset_session_selection(session_handles);
        error.set(None);
        auth_handles.step.set(AutoPeerStep::ManageSessions);
    } else {
        save_persisted_sessions(&PersistedSessions::default());
        reset_session_selection(session_handles);
        error.set(None);
        auth_handles.step.set(AutoPeerStep::EnterAsn);
    }
}

#[derive(Clone)]
pub struct AutoPeerController {
    pub autopeer_site_href: UseStateHandle<String>,
    pub looking_glass_site_href: UseStateHandle<String>,
    pub oidc_methods: UseStateHandle<Vec<AuthMethod>>,
    pub step: UseStateHandle<AutoPeerStep>,
    pub asn: UseStateHandle<String>,
    pub challenge_text: UseStateHandle<Option<String>>,
    pub methods: UseStateHandle<Vec<AuthMethod>>,
    pub selected_method: UseStateHandle<Option<AuthMethod>>,
    pub auth_session: UseStateHandle<Option<AuthSessionResponse>>,
    pub host_session: UseStateHandle<Option<AuthSessionResponse>>,
    pub nodes: UseStateHandle<Vec<NodeView>>,
    pub sessions: UseStateHandle<Vec<SessionView>>,
    pub draft: UseStateHandle<SessionDraft>,
    pub touched_fields: UseStateHandle<BTreeSet<String>>,
    pub editing_node: UseStateHandle<Option<String>>,
    pub config_stage: UseStateHandle<PeerConfigStage>,
    pub retire_confirmation: UseStateHandle<bool>,
    pub operation: UseStateHandle<Option<OperationStatus>>,
    pub error: UseStateHandle<Option<String>>,
    pub loading: UseStateHandle<bool>,
    pub loading_message: UseStateHandle<Option<String>>,
    pub impersonate_asn: UseStateHandle<String>,
    pub impersonate_mnt: UseStateHandle<String>,
    pub ssh_signature: UseStateHandle<String>,
    pub selected_pgp_key: UseStateHandle<String>,
    pub pgp_public_key: UseStateHandle<String>,
    pub pgp_signed_message: UseStateHandle<String>,
    pub selected_email_maintainer: UseStateHandle<String>,
    pub registry_email_code: UseStateHandle<String>,
    pub registry_email_sent_to: UseStateHandle<Vec<String>>,
    pub on_asn_change: Callback<String>,
    pub on_submit_asn: Callback<MouseEvent>,
    pub on_asn_keydown: Callback<KeyboardEvent>,
    pub on_enter_oidc: Callback<AuthMethod>,
    pub on_select_method: Callback<AuthMethod>,
    pub on_select_method_back: Callback<MouseEvent>,
    pub on_verify_back: Callback<MouseEvent>,
    pub on_verify: Callback<MouseEvent>,
    pub on_selected_email_maintainer_change: Callback<String>,
    pub on_registry_email_code_change: Callback<String>,
    pub on_send_registry_email: Callback<MouseEvent>,
    pub on_refresh: Callback<MouseEvent>,
    pub on_logout: Callback<MouseEvent>,
    pub on_impersonate_asn_change: Callback<String>,
    pub on_impersonate_mnt_change: Callback<String>,
    pub on_impersonate: Callback<MouseEvent>,
    pub on_return_to_host: Callback<MouseEvent>,
    pub on_submit_session: Callback<MouseEvent>,
    pub on_retire_selected_session: Callback<MouseEvent>,
}

#[hook]
pub fn use_autopeer_controller(
    default_autopeer_home_href: String,
    default_looking_glass_href: String,
) -> AutoPeerController {
    let persisted_sessions = load_persisted_sessions().unwrap_or_default();
    let api_base = use_state(|| None::<String>);
    let autopeer_site_href = {
        let initial = default_autopeer_home_href;
        use_state(move || initial)
    };
    let looking_glass_site_href = {
        let initial = default_looking_glass_href;
        use_state(move || initial)
    };
    let oidc_methods = use_state(Vec::<AuthMethod>::new);
    let step = use_state(|| AutoPeerStep::LoadingConfig);
    let asn = use_state(String::new);
    let challenge_id = use_state(|| None::<String>);
    let challenge_text = use_state(|| None::<String>);
    let methods = use_state(Vec::<AuthMethod>::new);
    let selected_method = use_state(|| None::<AuthMethod>);
    let auth_session = {
        let initial = persisted_sessions.auth_session.clone();
        use_state(move || initial)
    };
    let host_session = {
        let initial = persisted_sessions.host_session.clone();
        use_state(move || initial)
    };
    let nodes = use_state(Vec::<NodeView>::new);
    let sessions = use_state(Vec::<SessionView>::new);
    let draft = use_state(SessionDraft::default);
    let touched_fields = use_state(BTreeSet::<String>::new);
    let editing_node = use_state(|| None::<String>);
    let config_stage = use_state(|| PeerConfigStage::SelectNode);
    let retire_confirmation = use_state(|| false);
    let operation = use_state(|| None::<OperationStatus>);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let loading_message = use_state(|| None::<String>);

    let impersonate_asn = use_state(String::new);
    let impersonate_mnt = use_state(String::new);
    let ssh_signature = use_state(String::new);
    let selected_pgp_key = use_state(String::new);
    let pgp_public_key = use_state(String::new);
    let pgp_signed_message = use_state(String::new);
    let selected_email_maintainer = use_state(String::new);
    let registry_email_code = use_state(String::new);
    let registry_email_sent_to = use_state(Vec::<String>::new);

    let session_handles = SessionHandles {
        asn: asn.clone(),
        nodes: nodes.clone(),
        sessions: sessions.clone(),
        draft: draft.clone(),
        editing_node: editing_node.clone(),
        config_stage: config_stage.clone(),
    };
    let auth_handles = AuthHandles {
        auth_session: auth_session.clone(),
        host_session: host_session.clone(),
        step: step.clone(),
    };
    let auth_flow_handles = AuthFlowHandles {
        challenge_id: challenge_id.clone(),
        challenge_text: challenge_text.clone(),
        methods: methods.clone(),
        selected_method: selected_method.clone(),
        selected_pgp_key: selected_pgp_key.clone(),
        ssh_signature: ssh_signature.clone(),
        pgp_public_key: pgp_public_key.clone(),
        pgp_signed_message: pgp_signed_message.clone(),
        selected_email_maintainer: selected_email_maintainer.clone(),
        registry_email_code: registry_email_code.clone(),
        registry_email_sent_to: registry_email_sent_to.clone(),
    };

    {
        let api_base = api_base.clone();
        let autopeer_site_href = autopeer_site_href.clone();
        let looking_glass_site_href = looking_glass_site_href.clone();
        let oidc_methods = oidc_methods.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let session_handles = session_handles.clone();
        let auth_handles = auth_handles.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match service::load_runtime_config().await {
                    Ok(config) => {
                        let api_url = config.autopeer_url.unwrap_or_default();
                        autopeer_site_href.set(configured_href(
                            config.autopeer_site_url.as_deref(),
                            (*autopeer_site_href).as_str(),
                        ));
                        looking_glass_site_href.set(configured_href(
                            config.looking_glass_url.as_deref(),
                            (*looking_glass_site_href).as_str(),
                        ));
                        oidc_methods.set(config.oidc_methods);
                        api_base.set(Some(api_url.clone()));

                        if let Some(message) = hash_param("oidc_error") {
                            clear_location_hash();
                            error.set(Some(message));
                            auth_handles.step.set(AutoPeerStep::EnterAsn);
                            return;
                        }

                        if let Some(message) = hash_param("email_error") {
                            clear_location_hash();
                            error.set(Some(message));
                            auth_handles.step.set(AutoPeerStep::EnterAsn);
                            return;
                        }

                        if let Some(token) = hash_param("email_token") {
                            start_loading(
                                &loading,
                                &loading_message,
                                "loading.email_login",
                            );
                            finish_redirected_auth_session(
                                &api_url,
                                service::complete_registry_email(&api_url, &token).await,
                                &session_handles,
                                &auth_handles,
                                &error,
                                &loading,
                                &loading_message,
                            )
                            .await;
                            return;
                        }

                        if let Some(state) = hash_param("oidc_state") {
                            start_loading(
                                &loading,
                                &loading_message,
                                "loading.oidc_login",
                            );
                            finish_redirected_auth_session(
                                &api_url,
                                service::complete_oidc(&api_url, &state).await,
                                &session_handles,
                                &auth_handles,
                                &error,
                                &loading,
                                &loading_message,
                            )
                            .await;
                            return;
                        }

                        let persisted = PersistedSessions {
                            auth_session: (*auth_handles.auth_session).clone(),
                            host_session: (*auth_handles.host_session).clone(),
                        };
                        restore_persisted_state(
                            &api_url,
                            persisted,
                            &session_handles,
                            &auth_handles,
                            &error,
                        )
                        .await;
                    }
                    Err(message) => {
                        error.set(Some(message));
                        auth_handles.step.set(AutoPeerStep::EnterAsn);
                    }
                }
            });
            || ()
        });
    }

    {
        let auth_session = auth_session.clone();
        let host_session = host_session.clone();

        use_effect_with(
            ((*auth_session).clone(), (*host_session).clone()),
            move |(auth, host)| {
                save_persisted_sessions(&PersistedSessions {
                    auth_session: auth.clone(),
                    host_session: host.clone(),
                });
                || ()
            },
        );
    }

    {
        let touched_fields = touched_fields.clone();
        use_effect_with((*step).clone(), move |step| {
            if *step != AutoPeerStep::ManageSessions {
                touched_fields.set(BTreeSet::new());
            }
            || ()
        });
    }

    {
        let touched_fields = touched_fields.clone();
        let draft_node = draft.node.clone();
        use_effect_with(((*editing_node).clone(), draft_node), move |_| {
            touched_fields.set(BTreeSet::new());
            || ()
        });
    }

    {
        let retire_confirmation = retire_confirmation.clone();
        let draft_node = draft.node.clone();
        use_effect_with(
            (
                (*step).clone(),
                (*editing_node).clone(),
                draft_node,
                *config_stage,
            ),
            move |_| {
                retire_confirmation.set(false);
                || ()
            },
        );
    }

    let refresh_sessions = {
        let api_base = api_base.clone();
        let auth_session = auth_session.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let session_handles = session_handles.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(auth_session) = (*auth_session).clone() else {
                return;
            };

            start_loading(
                &loading,
                &loading_message,
                "loading.fetch_sessions",
            );

            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let session_handles = session_handles.clone();

            spawn_local(async move {
                match service::list_sessions(&api_base, &auth_session.session_token).await {
                    Ok(response) => {
                        apply_session_list(response, &session_handles);
                        if session_handles.editing_node.is_none() {
                            session_handles
                                .config_stage
                                .set(PeerConfigStage::SelectNode);
                        }
                        error.set(None);
                    }
                    Err(message) => error.set(Some(message)),
                }
                clear_loading(&loading, &loading_message);
            });
        })
    };

    let poll_operation = {
        let api_base = api_base.clone();
        let auth_session = auth_session.clone();
        let operation = operation.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let session_handles = session_handles.clone();

        Callback::from(move |initial_operation: OperationStatus| {
            let Some(api_base) = (*api_base).clone() else {
                return;
            };
            let Some(auth_session) = (*auth_session).clone() else {
                return;
            };

            let operation = operation.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let session_handles = session_handles.clone();

            spawn_local(async move {
                let mut current = initial_operation;
                operation.set(Some(current.clone()));

                loop {
                    if current.state.is_terminal() {
                        start_loading(
                            &loading,
                            &loading_message,
                            "loading.refresh_sessions",
                        );
                        match service::list_sessions(&api_base, &auth_session.session_token).await {
                            Ok(response) => {
                                apply_session_list(response, &session_handles);
                                reset_session_selection(&session_handles);
                                error.set(None);
                            }
                            Err(message) => error.set(Some(message)),
                        }
                        clear_loading(&loading, &loading_message);
                        break;
                    }

                    TimeoutFuture::new(3_000).await;
                    match service::get_operation(
                        &api_base,
                        &auth_session.session_token,
                        &current.id,
                    )
                    .await
                    {
                        Ok(updated) => {
                            current = updated.clone();
                            operation.set(Some(updated));
                        }
                        Err(message) => {
                            error.set(Some(message));
                            break;
                        }
                    }
                }
            });
        })
    };

    let on_asn_change = {
        let asn = asn.clone();
        Callback::from(move |value: String| asn.set(value))
    };

    let on_impersonate_asn_change = {
        let impersonate_asn = impersonate_asn.clone();
        Callback::from(move |value: String| impersonate_asn.set(value))
    };

    let on_impersonate_mnt_change = {
        let impersonate_mnt = impersonate_mnt.clone();
        Callback::from(move |value: String| impersonate_mnt.set(value))
    };

    let on_selected_email_maintainer_change = {
        let selected_email_maintainer = selected_email_maintainer.clone();
        Callback::from(move |value: String| selected_email_maintainer.set(value))
    };

    let on_registry_email_code_change = {
        let registry_email_code = registry_email_code.clone();
        Callback::from(move |value: String| registry_email_code.set(value))
    };

    let submit_asn = {
        let api_base = api_base.clone();
        let asn = asn.clone();
        let auth_flow_handles = auth_flow_handles.clone();
        let oidc_methods = oidc_methods.clone();
        let auth_handles = auth_handles.clone();
        let operation = operation.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let session_handles = session_handles.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };

            let asn_value = asn.trim().to_string();
            if asn_value.is_empty() {
                error.set(Some("error.asn_required".to_string()));
                return;
            }

            start_loading(
                &loading,
                &loading_message,
                "loading.fetch_methods",
            );
            error.set(None);
            operation.set(None);

            let auth_flow_handles = auth_flow_handles.clone();
            let oidc_methods = oidc_methods.clone();
            let auth_handles = auth_handles.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let session_handles = session_handles.clone();

            spawn_local(async move {
                match service::start_auth(&api_base, &asn_value).await {
                    Ok(response) => {
                        set_auth_challenge(
                            &auth_flow_handles,
                            response.challenge_id,
                            response.challenge_text,
                            filter_supported_methods(response.methods, !oidc_methods.is_empty()),
                        );
                        auth_handles.auth_session.set(None);
                        auth_handles.host_session.set(None);
                        reset_loaded_sessions(&session_handles, &asn_value);
                        auth_handles.step.set(AutoPeerStep::SelectMethod);
                    }
                    Err(message) => error.set(Some(message)),
                }
                clear_loading(&loading, &loading_message);
            });
        })
    };

    let on_submit_asn = {
        let submit_asn = submit_asn.clone();
        Callback::from(move |_| submit_asn.emit(()))
    };

    let on_asn_keydown = {
        let submit_asn = submit_asn.clone();
        let loading = loading.clone();
        let asn = asn.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Enter" && !*loading && !asn.trim().is_empty() {
                event.prevent_default();
                submit_asn.emit(());
            }
        })
    };

    let on_enter_oidc = {
        let api_base = api_base.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();

        Callback::from(move |method: AuthMethod| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(provider) = method.provider.clone() else {
                error.set(Some("error.oidc_provider_missing".to_string()));
                return;
            };

            start_loading(
                &loading,
                &loading_message,
                "loading.redirect_oidc",
            );
            error.set(None);

            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let error = error.clone();

            spawn_local(async move {
                match service::start_oidc(&api_base, &provider, None).await {
                    Ok(response) => {
                        if let Err(message) = redirect_to(&response.authorization_url) {
                            clear_loading(&loading, &loading_message);
                            error.set(Some(message));
                        }
                    }
                    Err(message) => {
                        clear_loading(&loading, &loading_message);
                        error.set(Some(message));
                    }
                }
            });
        })
    };

    let on_select_method_back = {
        let auth_flow_handles = auth_flow_handles.clone();
        let auth_handles = auth_handles.clone();
        Callback::from(move |_| {
            clear_auth_challenge(&auth_flow_handles);
            auth_handles.step.set(AutoPeerStep::EnterAsn);
        })
    };

    let on_select_method = {
        let api_base = api_base.clone();
        let asn = asn.clone();
        let auth_flow_handles = auth_flow_handles.clone();
        let oidc_methods = oidc_methods.clone();
        let auth_handles = auth_handles.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();

        Callback::from(move |method_value: AuthMethod| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };

            let asn_value = asn.trim().to_string();
            if asn_value.is_empty() {
                error.set(Some("error.asn_required".to_string()));
                return;
            }

            start_loading(
                &loading,
                &loading_message,
                "loading.fetch_challenge",
            );
            error.set(None);

            let auth_flow_handles = auth_flow_handles.clone();
            let auth_handles = auth_handles.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let oidc_methods = oidc_methods.clone();

            spawn_local(async move {
                match service::start_auth(&api_base, &asn_value).await {
                    Ok(response) => {
                        let available_methods =
                            filter_supported_methods(response.methods, !oidc_methods.is_empty());
                        let matched_method =
                            matching_auth_method(&available_methods, &method_value);

                        match matched_method {
                            Some(method) => {
                                set_auth_challenge(
                                    &auth_flow_handles,
                                    response.challenge_id,
                                    response.challenge_text,
                                    available_methods,
                                );
                                set_selected_auth_method(&auth_flow_handles, method);
                                auth_handles.step.set(AutoPeerStep::VerifyMethod);
                            }
                            None => error.set(Some(
                                "error.method_no_longer_available"
                                    .to_string(),
                            )),
                        }
                    }
                    Err(message) => error.set(Some(message)),
                }
                clear_loading(&loading, &loading_message);
            });
        })
    };

    let on_verify_back = {
        let step = step.clone();
        let error = error.clone();
        Callback::from(move |_| {
            error.set(None);
            step.set(AutoPeerStep::SelectMethod);
        })
    };

    let on_send_registry_email = {
        let api_base = api_base.clone();
        let challenge_id = challenge_id.clone();
        let selected_method = selected_method.clone();
        let selected_email_maintainer = selected_email_maintainer.clone();
        let registry_email_sent_to = registry_email_sent_to.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(challenge_id) = (*challenge_id).clone() else {
                error.set(Some("error.missing_challenge_id".to_string()));
                return;
            };
            let Some(method) = (*selected_method).clone() else {
                error.set(Some("error.choose_method_first".to_string()));
                return;
            };
            if method.kind != AuthMethodKind::RegistryEmail {
                error.set(Some(
                    "error.email_auth_inactive".to_string(),
                ));
                return;
            }

            let selected_target =
                selected_registry_email_target(&method, selected_email_maintainer.as_str());
            let Some(target) = selected_target else {
                error.set(Some(
                    "error.email_choose_maintainer".to_string(),
                ));
                return;
            };

            start_loading(
                &loading,
                &loading_message,
                "loading.send_email",
            );
            error.set(None);

            let effective_mnt = target.maintainer.clone();
            let registry_email_sent_to = registry_email_sent_to.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();

            spawn_local(async move {
                match service::send_registry_email(&api_base, &challenge_id, Some(&effective_mnt))
                    .await
                {
                    Ok(response) => {
                        registry_email_sent_to.set(response.emails);
                        error.set(None);
                    }
                    Err(message) => error.set(Some(message)),
                }

                clear_loading(&loading, &loading_message);
            });
        })
    };

    let on_verify = {
        let api_base = api_base.clone();
        let challenge_id = challenge_id.clone();
        let selected_method = selected_method.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let ssh_signature = ssh_signature.clone();
        let pgp_public_key = pgp_public_key.clone();
        let pgp_signed_message = pgp_signed_message.clone();
        let registry_email_code = registry_email_code.clone();
        let session_handles = session_handles.clone();
        let auth_handles = auth_handles.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(challenge_id) = (*challenge_id).clone() else {
                error.set(Some("error.missing_challenge_id".to_string()));
                return;
            };
            let Some(method) = (*selected_method).clone() else {
                error.set(Some("error.choose_method_first".to_string()));
                return;
            };
            if method.kind == AuthMethodKind::RegistrySsh
                && let Err(message) = validate_ssh_signature_input(ssh_signature.as_str())
            {
                error.set(Some(message.to_string()));
                return;
            }
            if method.kind == AuthMethodKind::RegistryEmail && registry_email_code.trim().is_empty()
            {
                error.set(Some(
                    "error.enter_email_code".to_string(),
                ));
                return;
            }

            let loading_text = match method.kind {
                AuthMethodKind::RegistrySsh => {
                    "loading.check_ssh"
                }
                AuthMethodKind::RegistryPgp => {
                    "loading.check_pgp"
                }
                AuthMethodKind::RegistryEmail => "loading.check_email",
                AuthMethodKind::Oidc => "loading.redirect_oidc",
                AuthMethodKind::HostImpersonation => "loading.host_session_prep",
            };
            start_loading(&loading, &loading_message, loading_text);
            error.set(None);

            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let ssh_signature_value = (*ssh_signature).clone();
            let pgp_public_key_value = (*pgp_public_key).clone();
            let pgp_signed_message_value = (*pgp_signed_message).clone();
            let registry_email_code_value = (*registry_email_code).clone();
            let session_handles = session_handles.clone();
            let auth_handles = auth_handles.clone();

            spawn_local(async move {
                if method.kind == AuthMethodKind::Oidc {
                    let Some(provider) = method.provider.clone() else {
                        clear_loading(&loading, &loading_message);
                        error.set(Some("error.oidc_provider_missing".to_string()));
                        return;
                    };

                    match service::start_oidc(&api_base, &provider, Some(&challenge_id)).await {
                        Ok(response) => {
                            if let Err(message) = redirect_to(&response.authorization_url) {
                                clear_loading(&loading, &loading_message);
                                error.set(Some(message));
                            }
                        }
                        Err(message) => {
                            clear_loading(&loading, &loading_message);
                            error.set(Some(message));
                        }
                    }
                    return;
                }

                let result = match method.kind {
                    AuthMethodKind::RegistrySsh => {
                        service::verify_registry_ssh(&api_base, &challenge_id, &ssh_signature_value)
                            .await
                    }
                    AuthMethodKind::RegistryPgp => {
                        service::verify_registry_pgp(
                            &api_base,
                            &challenge_id,
                            &pgp_public_key_value,
                            &pgp_signed_message_value,
                        )
                        .await
                    }
                    AuthMethodKind::RegistryEmail => {
                        service::verify_registry_email(
                            &api_base,
                            &challenge_id,
                            &registry_email_code_value,
                        )
                        .await
                    }
                    AuthMethodKind::Oidc => unreachable!(),
                    AuthMethodKind::HostImpersonation => {
                        clear_loading(&loading, &loading_message);
                        error.set(Some(
                            "error.impersonate_after_host"
                                .to_string(),
                        ));
                        return;
                    }
                };

                match result {
                    Ok(session) => {
                        activate_authenticated_session(
                            &api_base,
                            session,
                            &session_handles,
                            &auth_handles,
                            &error,
                        )
                        .await;
                    }
                    Err(message) => error.set(Some(message)),
                }

                clear_loading(&loading, &loading_message);
            });
        })
    };

    let on_refresh = {
        let refresh_sessions = refresh_sessions.clone();
        Callback::from(move |_| refresh_sessions.emit(()))
    };

    let on_logout = {
        let auth_flow_handles = auth_flow_handles.clone();
        let auth_handles = auth_handles.clone();
        let session_handles = session_handles.clone();
        let operation = operation.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let impersonate_asn = impersonate_asn.clone();
        let impersonate_mnt = impersonate_mnt.clone();

        Callback::from(move |_| {
            save_persisted_sessions(&PersistedSessions::default());
            clear_auth_challenge(&auth_flow_handles);
            auth_handles.auth_session.set(None);
            auth_handles.host_session.set(None);
            clear_session_state(&session_handles);
            operation.set(None);
            error.set(None);
            clear_loading(&loading, &loading_message);
            auth_handles.step.set(AutoPeerStep::EnterAsn);
            clear_impersonation_inputs(&impersonate_asn, &impersonate_mnt);
        })
    };

    let on_impersonate = {
        let api_base = api_base.clone();
        let auth_session = auth_session.clone();
        let host_session = host_session.clone();
        let impersonate_asn = impersonate_asn.clone();
        let impersonate_mnt = impersonate_mnt.clone();
        let operation = operation.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let session_handles = session_handles.clone();
        let auth_handles = auth_handles.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(base_session) = (*host_session).clone().or_else(|| {
                (*auth_session)
                    .clone()
                    .filter(|session| session.can_impersonate)
            }) else {
                error.set(Some(
                    "error.host_auth_first"
                        .to_string(),
                ));
                return;
            };

            let target_asn = impersonate_asn.trim().to_string();
            if target_asn.is_empty() {
                error.set(Some("error.enter_impersonate_asn".to_string()));
                return;
            }

            start_loading(
                &loading,
                &loading_message,
                "loading.authing_asn",
            );
            error.set(None);
            operation.set(None);

            let impersonate_mnt_value = (*impersonate_mnt).clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let session_handles = session_handles.clone();
            let auth_handles = auth_handles.clone();

            spawn_local(async move {
                match service::impersonate_asn(
                    &api_base,
                    &base_session.session_token,
                    &target_asn,
                    Some(impersonate_mnt_value.trim()),
                )
                .await
                {
                    Ok(session) => {
                        activate_authenticated_session(
                            &api_base,
                            session,
                            &session_handles,
                            &auth_handles,
                            &error,
                        )
                        .await;
                    }
                    Err(message) => error.set(Some(message)),
                }

                clear_loading(&loading, &loading_message);
            });
        })
    };

    let on_return_to_host = {
        let api_base = api_base.clone();
        let host_session = host_session.clone();
        let auth_session = auth_session.clone();
        let session_handles = session_handles.clone();
        let impersonate_asn = impersonate_asn.clone();
        let impersonate_mnt = impersonate_mnt.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(host_session_value) = (*host_session).clone() else {
                error.set(Some(
                    "error.no_host_session".to_string(),
                ));
                return;
            };

            start_loading(
                &loading,
                &loading_message,
                "loading.restore_host",
            );
            error.set(None);

            let auth_session = auth_session.clone();
            let session_handles = session_handles.clone();
            let impersonate_asn = impersonate_asn.clone();
            let impersonate_mnt = impersonate_mnt.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();

            spawn_local(async move {
                match service::list_sessions(&api_base, &host_session_value.session_token).await {
                    Ok(response) => {
                        auth_session.set(Some(host_session_value.clone()));
                        apply_session_list(response, &session_handles);
                        let next_draft = sync_create_draft(
                            &host_session_value.asn,
                            &session_handles.nodes,
                            &session_handles.sessions,
                            &session_handles.draft,
                        );
                        session_handles.draft.set(next_draft);
                        reset_session_selection(&session_handles);
                        clear_impersonation_inputs(&impersonate_asn, &impersonate_mnt);
                        error.set(None);
                    }
                    Err(message) => {
                        session_handles.asn.set(host_session_value.asn.clone());
                        error.set(Some(message));
                    }
                }

                clear_loading(&loading, &loading_message);
            });
        })
    };

    let on_submit_session = {
        let api_base = api_base.clone();
        let auth_session = auth_session.clone();
        let draft = draft.clone();
        let editing_node = editing_node.clone();
        let operation = operation.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let poll_operation = poll_operation.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(auth_session) = (*auth_session).clone() else {
                error.set(Some("error.authenticate_first".to_string()));
                return;
            };

            let draft_value = (*draft).clone();
            if editing_node.is_none() && draft_value.node.trim().is_empty() {
                error.set(Some(
                    "error.choose_node_inline".to_string(),
                ));
                return;
            }
            let spec = match draft_value.to_spec() {
                Ok(spec) => spec,
                Err(message) => {
                    error.set(Some(message));
                    return;
                }
            };

            start_loading(
                &loading,
                &loading_message,
                if editing_node.is_some() {
                    "loading.update_pr"
                } else {
                    "loading.create_pr"
                },
            );
            error.set(None);

            let operation = operation.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let poll_operation = poll_operation.clone();
            let editing = (*editing_node).clone();
            let session_asn = auth_session.asn.clone();

            spawn_local(async move {
                let result = if let Some(node) = editing {
                    service::update_session(
                        &api_base,
                        &auth_session.session_token,
                        &node,
                        &session_asn,
                        &UpdateSessionRequest {
                            session: spec,
                        },
                    )
                    .await
                } else {
                    service::create_session(
                        &api_base,
                        &auth_session.session_token,
                        &CreateSessionRequest {
                            node: draft_value.node.clone(),
                            session: spec,
                        },
                    )
                    .await
                };

                match result {
                    Ok(status) => {
                        operation.set(Some(status.clone()));
                        poll_operation.emit(status);
                    }
                    Err(message) => error.set(Some(message)),
                }

                clear_loading(&loading, &loading_message);
            });
        })
    };

    let on_retire_selected_session = {
        let api_base = api_base.clone();
        let auth_session = auth_session.clone();
        let draft = draft.clone();
        let sessions = sessions.clone();
        let editing_node = editing_node.clone();
        let operation = operation.clone();
        let error = error.clone();
        let loading = loading.clone();
        let loading_message = loading_message.clone();
        let poll_operation = poll_operation.clone();
        let config_stage = config_stage.clone();
        let retire_confirmation = retire_confirmation.clone();

        Callback::from(move |_| {
            let Some(api_base) = require_api_base(&api_base, &error) else {
                return;
            };
            let Some(auth_session) = (*auth_session).clone() else {
                error.set(Some("error.authenticate_first".to_string()));
                return;
            };
            let selected_node = selected_session_node_name((*editing_node).as_deref(), &draft)
                .and_then(|node| session_for_node(&node, &sessions).map(|_| node));
            let Some(node) = selected_node else {
                error.set(Some(
                    "error.choose_managed_to_retire".to_string(),
                ));
                return;
            };
            if !*retire_confirmation {
                retire_confirmation.set(true);
                error.set(None);
                return;
            }

            retire_confirmation.set(false);
            editing_node.set(None);
            config_stage.set(PeerConfigStage::SelectNode);

            start_loading(
                &loading,
                &loading_message,
                "loading.retire_pr",
            );
            error.set(None);

            let operation = operation.clone();
            let error = error.clone();
            let loading = loading.clone();
            let loading_message = loading_message.clone();
            let poll_operation = poll_operation.clone();
            let session_asn = auth_session.asn.clone();

            spawn_local(async move {
                match service::delete_session(
                    &api_base,
                    &auth_session.session_token,
                    &node,
                    &session_asn,
                )
                .await
                {
                    Ok(status) => {
                        operation.set(Some(status.clone()));
                        poll_operation.emit(status);
                    }
                    Err(message) => error.set(Some(message)),
                }

                clear_loading(&loading, &loading_message);
            });
        })
    };

    AutoPeerController {
        autopeer_site_href,
        looking_glass_site_href,
        oidc_methods,
        step,
        asn,
        challenge_text,
        methods,
        selected_method,
        auth_session,
        host_session,
        nodes,
        sessions,
        draft,
        touched_fields,
        editing_node,
        config_stage,
        retire_confirmation,
        operation,
        error,
        loading,
        loading_message,
        impersonate_asn,
        impersonate_mnt,
        ssh_signature,
        selected_pgp_key,
        pgp_public_key,
        pgp_signed_message,
        selected_email_maintainer,
        registry_email_code,
        registry_email_sent_to,
        on_asn_change,
        on_submit_asn,
        on_asn_keydown,
        on_enter_oidc,
        on_select_method,
        on_select_method_back,
        on_verify_back,
        on_verify,
        on_selected_email_maintainer_change,
        on_registry_email_code_change,
        on_send_registry_email,
        on_refresh,
        on_logout,
        on_impersonate_asn_change,
        on_impersonate_mnt_change,
        on_impersonate,
        on_return_to_host,
        on_submit_session,
        on_retire_selected_session,
    }
}

#[cfg(test)]
mod tests {
    use common::auto_peer::{AuthMethod, AuthMethodKind};

    use crate::controller::{
        configured_href, filter_supported_methods, matching_auth_method,
        validate_ssh_signature_input,
    };

    #[test]
    fn prefers_runtime_configured_link_over_fallback() {
        assert_eq!(
            configured_href(Some("https://network.owo.li"), "https://lg.owo.li/"),
            "https://network.owo.li"
        );
    }

    #[test]
    fn hides_oidc_methods_when_runtime_config_disables_them() {
        let methods = vec![
            AuthMethod {
                kind: AuthMethodKind::RegistrySsh,
                label: "Registry SSH".into(),
                description: "SSH".into(),
                ..AuthMethod::default()
            },
            AuthMethod {
                kind: AuthMethodKind::Oidc,
                label: "Kioubit".into(),
                description: "OIDC".into(),
                provider: Some("kioubit".into()),
                ..AuthMethod::default()
            },
        ];

        let filtered = filter_supported_methods(methods, false);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].kind, AuthMethodKind::RegistrySsh);
    }

    #[test]
    fn rejects_raw_challenge_text_in_ssh_signature_field() {
        assert_eq!(
            validate_ssh_signature_input(
                "dn42-autopeer challenge\nasn: 4242421024\nchallenge_id: example\nissued_at: 2026-04-18T12:42:04.075Z"
            ),
            Err("error.ssh.unsigned_challenge"),
        );
    }

    #[test]
    fn accepts_armored_ssh_signature_blocks() {
        assert_eq!(
            validate_ssh_signature_input(
                "-----BEGIN SSH SIGNATURE-----\nZm9v\n-----END SSH SIGNATURE-----"
            ),
            Ok(()),
        );
    }

    #[test]
    fn matches_auth_method_by_kind_and_provider() {
        let registry = AuthMethod {
            kind: AuthMethodKind::RegistrySsh,
            label: "Registry SSH".into(),
            description: "SSH".into(),
            ..AuthMethod::default()
        };
        let kioubit = AuthMethod {
            kind: AuthMethodKind::Oidc,
            label: "Kioubit".into(),
            description: "OIDC".into(),
            provider: Some("kioubit".into()),
            ..AuthMethod::default()
        };
        let lwm = AuthMethod {
            kind: AuthMethodKind::Oidc,
            label: "LWM".into(),
            description: "OIDC".into(),
            provider: Some("lwm".into()),
            ..AuthMethod::default()
        };
        let available_methods = vec![registry.clone(), kioubit.clone(), lwm];

        assert_eq!(
            matching_auth_method(&available_methods, &kioubit),
            Some(kioubit),
        );
        assert_eq!(
            matching_auth_method(
                &available_methods,
                &AuthMethod {
                    kind: AuthMethodKind::Oidc,
                    provider: Some("missing".into()),
                    ..AuthMethod::default()
                }
            ),
            None,
        );
        assert_eq!(
            matching_auth_method(&available_methods, &registry),
            Some(registry),
        );
    }
}
