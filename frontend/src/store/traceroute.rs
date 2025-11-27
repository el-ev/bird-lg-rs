use crate::models::TracerouteHop;

#[derive(Clone, Debug, PartialEq)]
pub enum NodeTracerouteResult {
    Hops(Vec<TracerouteHop>),
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TracerouteState {
    pub target: String,
    pub node: String,
    pub version: String,
    pub loading: bool,
    pub error: Option<String>,
    pub results: Vec<(String, NodeTracerouteResult)>,
    pub last_target: String,
    pub last_version: String,
}

impl Default for TracerouteState {
    fn default() -> Self {
        Self {
            target: String::new(),
            node: String::new(),
            version: "auto".to_string(),
            loading: false,
            error: None,
            results: Vec::new(),
            last_target: String::new(),
            last_version: String::new(),
        }
    }
}

pub enum TracerouteAction {
    SetTarget(String),
    SetNode(String),
    SetVersion(String),
    SetError(String),
    ClearError,
    Start,
    End,
    InitResult(String),
    UpdateResult(String, NodeTracerouteResult),
    SetLastParams(String, String), // target, version
}

impl TracerouteState {
    pub fn reduce(&mut self, action: TracerouteAction) {
        match action {
            TracerouteAction::SetTarget(target) => {
                self.target = target;
                self.error = None;
            }
            TracerouteAction::SetNode(node) => {
                self.node = node;
            }
            TracerouteAction::SetVersion(version) => {
                self.version = version;
            }
            TracerouteAction::SetError(err) => {
                self.error = Some(err);
            }
            TracerouteAction::ClearError => {
                self.error = None;
            }
            TracerouteAction::Start => {
                self.loading = true;
                self.results.clear();
            }
            TracerouteAction::End => {
                self.loading = false;
            }
            TracerouteAction::InitResult(node) => {
                self.results.retain(|(n, _)| n != &node);
                self.results
                    .push((node, NodeTracerouteResult::Hops(Vec::new())));
            }
            TracerouteAction::UpdateResult(node, result) => {
                let (_, existing_result) = self
                    .results
                    .iter_mut()
                    .find(|(n, _)| n == &node)
                    .expect("UpdateResult called for an uninitialized node");

                match (existing_result, result) {
                    (NodeTracerouteResult::Hops(hops), NodeTracerouteResult::Hops(new_hops)) => {
                        hops.extend(new_hops);
                    }
                    (ex @ NodeTracerouteResult::Hops(_), e @ NodeTracerouteResult::Error(_)) => {
                        *ex = e;
                    }
                    _ => unreachable!(),
                }
            }
            TracerouteAction::SetLastParams(target, version) => {
                self.last_target = target;
                self.last_version = version;
            }
        }
    }
}
