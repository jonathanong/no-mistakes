use crate::codebase::testing_analysis::dynamic_imports::SetupReachability;
pub(super) use crate::codebase::testing_analysis::dynamic_imports::Rejected;

/// Shapes that may register setup coverage or load another setup helper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Classified {
    pub(super) accepted_mocks: Vec<String>,
    pub(super) accepted_requires: Vec<String>,
    pub(super) rejected: Vec<Rejected>,
}

impl From<SetupReachability> for Classified {
    fn from(reachability: SetupReachability) -> Self {
        Self {
            accepted_mocks: reachability.accepted_mocks,
            accepted_requires: reachability.accepted_requires,
            rejected: reachability.rejected,
        }
    }
}
