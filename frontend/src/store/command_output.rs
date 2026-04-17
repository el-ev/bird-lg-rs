use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandOutputKind {
    Ping,
    RouteLookup,
    ProtocolDetails,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandOutputSession {
    pub kind: CommandOutputKind,
    pub command: String,
    pub content: String,
    pub complete: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CommandOutputState {
    pub active_request_id: Option<String>,
    pub sessions: HashMap<String, CommandOutputSession>,
}

impl CommandOutputState {
    pub fn start(&mut self, request_id: String, kind: CommandOutputKind, command: String) {
        self.active_request_id = Some(request_id.clone());
        self.sessions.insert(
            request_id,
            CommandOutputSession {
                kind,
                command,
                content: "Loading...".to_string(),
                complete: false,
            },
        );
        self.prune_inactive_terminal();
    }

    pub fn initialize(&mut self, request_id: &str) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            session.content.clear();
        }
    }

    pub fn append_lines(&mut self, request_id: &str, lines: &[String]) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            append_output_lines(&mut session.content, lines);
        }
    }

    pub fn append_error(&mut self, request_id: &str, error: &str) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            if !session.content.is_empty() && !session.content.ends_with('\n') {
                session.content.push('\n');
            }
            session.content.push_str("Error: ");
            session.content.push_str(error);
            session.content.push('\n');
            session.complete = true;
        }
        self.prune_terminal_if_inactive(request_id);
    }

    pub fn finish(&mut self, request_id: &str) {
        if let Some(session) = self.sessions.get_mut(request_id) {
            session.complete = true;
        }
        self.prune_terminal_if_inactive(request_id);
    }

    pub fn close_active(&mut self) {
        if let Some(request_id) = self.active_request_id.take() {
            self.sessions.remove(&request_id);
        }
    }

    pub fn active_session(&self) -> Option<&CommandOutputSession> {
        self.active_request_id
            .as_ref()
            .and_then(|request_id| self.sessions.get(request_id))
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

fn append_output_lines(content: &mut String, lines: &[String]) {
    if lines.is_empty() {
        return;
    }

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    content.push_str(&lines.join("\n"));

    if !content.ends_with('\n') {
        content.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandOutputKind, CommandOutputState};

    #[test]
    fn output_updates_do_not_insert_blank_lines_between_batches() {
        let mut state = CommandOutputState::default();
        state.start(
            "req-1".to_string(),
            CommandOutputKind::Ping,
            "tester@edge-a$ ping -c 5 1.1.1.1".to_string(),
        );
        state.initialize("req-1");
        state.append_lines(
            "req-1",
            &["64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=1.09 ms".to_string()],
        );
        state.append_lines(
            "req-1",
            &["64 bytes from 1.1.1.1: icmp_seq=2 ttl=57 time=1.10 ms".to_string()],
        );

        let session = state.active_session().expect("active session");
        assert_eq!(
            session.content,
            "64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=1.09 ms\n64 bytes from 1.1.1.1: icmp_seq=2 ttl=57 time=1.10 ms\n"
        );
    }
}
