use std::collections::HashMap;

use common::traceroute::TracerouteHop;

#[derive(Clone, Debug, PartialEq)]
pub enum TracerouteResult {
    Hops(Vec<TracerouteHop>),
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TracerouteSession {
    pub target: String,
    pub version: String,
    pub pending: usize,
    pub complete: bool,
    pub results: Vec<(String, TracerouteResult)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TracerouteState {
    pub active_request_id: Option<String>,
    pub sessions: HashMap<String, TracerouteSession>,
}

impl TracerouteState {
    pub fn start(&mut self, request_id: String, target: String, version: String, pending: usize) {
        self.active_request_id = Some(request_id.clone());
        self.sessions.insert(
            request_id,
            TracerouteSession {
                target,
                version,
                pending,
                complete: pending == 0,
                results: Vec::new(),
            },
        );
        self.prune_inactive_terminal();
    }

    pub fn initialize(&mut self, request_id: &str, node: String) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            ensure_traceroute_result(&mut session.results, node);
        }
    }

    pub fn update(&mut self, request_id: &str, node: String, result: TracerouteResult) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            let existing_result = ensure_traceroute_result(&mut session.results, node);

            match (existing_result, result) {
                (TracerouteResult::Hops(hops), TracerouteResult::Hops(new_hops)) => {
                    hops.extend(new_hops);
                }
                (existing, error @ TracerouteResult::Error(_)) => {
                    *existing = error;
                }
                (TracerouteResult::Error(_), TracerouteResult::Hops(_)) => {}
            }
        }
    }

    pub fn finish_one(&mut self, request_id: &str) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            if session.pending > 0 {
                session.pending -= 1;
            }
            session.complete = session.pending == 0;
        }
        self.prune_terminal_if_inactive(request_id);
    }

    pub fn active_session(&self) -> Option<&TracerouteSession> {
        self.active_request_id
            .as_ref()
            .and_then(|request_id| self.sessions.get(request_id))
    }

    pub fn is_loading(&self) -> bool {
        self.active_session()
            .is_some_and(|session| !session.complete)
    }

    fn prune_terminal_if_inactive(&mut self, request_id: &str) {
        let is_active = self.active_request_id.as_deref() == Some(request_id);
        let is_terminal = self
            .sessions
            .get(request_id)
            .is_some_and(|session| session.complete);

        if !is_active && is_terminal {
            self.sessions.remove(request_id);
        }
    }

    fn prune_inactive_terminal(&mut self) {
        let active_request_id = self.active_request_id.clone();
        self.sessions.retain(|request_id, session| {
            !session.complete || active_request_id.as_deref() == Some(request_id.as_str())
        });
    }
}

fn ensure_traceroute_result(
    results: &mut Vec<(String, TracerouteResult)>,
    node: String,
) -> &mut TracerouteResult {
    if let Some(idx) = results
        .iter()
        .position(|(existing_node, _)| existing_node == &node)
    {
        return &mut results[idx].1;
    }

    results.push((node, TracerouteResult::Hops(Vec::new())));
    &mut results
        .last_mut()
        .expect("new traceroute result inserted")
        .1
}
