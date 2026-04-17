use common::traceroute::TracerouteHop;

#[derive(Clone, Debug, PartialEq)]
pub enum TracerouteResult {
    Hops(Vec<TracerouteHop>),
    Error(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TracerouteState {
    pub target: String,
    pub node: String,
    pub version: String,
    pub loading: bool,
    pub error: Option<String>,
    pub results: Vec<(String, TracerouteResult)>,
    pub last_target: String,
    pub last_version: String,
    pub pending: usize,
}

pub enum TracerouteAction {
    SetTarget(String),
    SetNode(String),
    SetVersion(String),
    SetError(String),
    ClearError,
    Start(usize),
    EndOne,
    InitResult(String),
    UpdateResult(String, TracerouteResult),
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
            TracerouteAction::Start(pending) => {
                self.loading = pending > 0;
                self.pending = pending;
                self.results.clear();
            }
            TracerouteAction::EndOne => {
                if self.pending > 0 {
                    self.pending -= 1;
                }
                self.loading = self.pending > 0;
            }
            TracerouteAction::InitResult(node) => {
                self.results.retain(|(n, _)| n != &node);
                self.results
                    .push((node, TracerouteResult::Hops(Vec::new())));
            }
            TracerouteAction::UpdateResult(node, result) => {
                let (_, existing_result) = self
                    .results
                    .iter_mut()
                    .find(|(n, _)| n == &node)
                    .expect("UpdateResult called for an uninitialized node");

                match (existing_result, result) {
                    (TracerouteResult::Hops(hops), TracerouteResult::Hops(new_hops)) => {
                        hops.extend(new_hops);
                    }
                    (ex @ TracerouteResult::Hops(_), e @ TracerouteResult::Error(_)) => {
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
