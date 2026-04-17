use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum PingResult {
    Lines(Vec<String>),
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PingSession {
    pub node: String,
    pub target: String,
    pub version: String,
    pub complete: bool,
    pub results: Vec<(String, PingResult)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PingState {
    pub active_request_id: Option<String>,
    pub sessions: HashMap<String, PingSession>,
}

impl PingState {
    pub fn start(&mut self, request_id: String, node: String, target: String, version: String) {
        self.active_request_id = Some(request_id.clone());
        self.sessions.insert(
            request_id,
            PingSession {
                node,
                target,
                version,
                complete: false,
                results: Vec::new(),
            },
        );
        self.prune_inactive_terminal();
    }

    pub fn initialize(&mut self, request_id: &str, node: String) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            ensure_ping_result(&mut session.results, node);
        }
    }

    pub fn update(&mut self, request_id: &str, node: String, result: PingResult) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            let existing_result = ensure_ping_result(&mut session.results, node);

            match (existing_result, result) {
                (PingResult::Lines(lines), PingResult::Lines(new_lines)) => {
                    lines.extend(new_lines);
                }
                (existing, error @ PingResult::Error(_)) => {
                    *existing = error;
                }
                (PingResult::Error(_), PingResult::Lines(_)) => {}
            }
        }
    }

    pub fn finish(&mut self, request_id: &str) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            session.complete = true;
        }
        self.prune_terminal_if_inactive(request_id);
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

fn ensure_ping_result<'a>(
    results: &'a mut Vec<(String, PingResult)>,
    node: String,
) -> &'a mut PingResult {
    if let Some(idx) = results
        .iter()
        .position(|(existing_node, _)| existing_node == &node)
    {
        return &mut results[idx].1;
    }

    results.push((node, PingResult::Lines(Vec::new())));
    &mut results.last_mut().expect("new ping result inserted").1
}
