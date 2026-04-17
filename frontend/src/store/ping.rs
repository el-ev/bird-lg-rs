#[derive(Clone, Debug, PartialEq)]
pub enum PingResult {
    Lines(Vec<String>),
    Error(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PingState {
    pub target: String,
    pub node: String,
    pub version: String,
    pub loading: bool,
    pub error: Option<String>,
    pub results: Vec<(String, PingResult)>,
    pub last_target: String,
    pub last_version: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PingAction {
    SetTarget(String),
    SetNode(String),
    SetVersion(String),
    SetError(String),
    ClearError,
    Start,
    End,
    InitResult(String),
    UpdateResult(String, PingResult),
    SetLastParams(String, String), // target, version
}

impl PingState {
    pub fn reduce(&mut self, action: PingAction) {
        match action {
            PingAction::SetTarget(target) => {
                self.target = target;
                self.error = None;
            }
            PingAction::SetNode(node) => {
                self.node = node;
            }
            PingAction::SetVersion(version) => {
                self.version = version;
            }
            PingAction::SetError(err) => {
                self.error = Some(err);
            }
            PingAction::ClearError => {
                self.error = None;
            }
            PingAction::Start => {
                self.loading = true;
                self.results.clear();
            }
            PingAction::End => {
                self.loading = false;
            }
            PingAction::InitResult(node) => {
                self.results.retain(|(n, _)| n != &node);
                self.results.push((node, PingResult::Lines(Vec::new())));
            }
            PingAction::UpdateResult(node, result) => {
                let (_, existing_result) = self
                    .results
                    .iter_mut()
                    .find(|(n, _)| n == &node)
                    .expect("UpdateResult called for an uninitialized node");

                match (existing_result, result) {
                    (PingResult::Lines(lines), PingResult::Lines(new_lines)) => {
                        lines.extend(new_lines);
                    }
                    (ex @ PingResult::Lines(_), e @ PingResult::Error(_)) => {
                        *ex = e;
                    }
                    _ => unreachable!(),
                }
            }
            PingAction::SetLastParams(target, version) => {
                self.last_target = target;
                self.last_version = version;
            }
        }
    }
}
