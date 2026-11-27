use super::*;

#[path = "escaped_array_candidates.rs"]
mod candidates;
#[path = "escaped_array_group.rs"]
mod group;
#[path = "escaped_array_hash.rs"]
mod hash;
#[path = "escaped_array_index.rs"]
mod index;
pub(in super::super) use index::ReplayIndex;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(in super::super) struct Recipient {
    pub(in super::super) owners: Vec<CallableId>,
    pub(in super::super) position: usize,
    pub(in super::super) open: Option<usize>,
}
pub(in super::super) struct Projection {
    pub(in super::super) baseline: Vec<(usize, ValueExpression)>,
    pub(in super::super) additional: Vec<(usize, ValueExpression)>,
    pub(in super::super) minimum: usize,
    pub(in super::super) baseline_length: usize,
    pub(in super::super) has_alternatives: bool,
}

#[cfg(feature = "test-instrumentation")]
#[path = "escaped_array_replay_metrics.rs"]
mod metrics;
#[cfg(feature = "test-instrumentation")]
pub(super) use metrics::record;
#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "escaped_array_replay_tests.rs"]
mod tests;
