use super::*;

#[derive(Default)]
pub(super) struct PhaseStats {
    pub(super) nodes: usize,
    pub(super) path_tokens: usize,
    pub(super) pending_tokens: usize,
    pub(super) queries: std::cell::Cell<usize>,
}
impl ActiveReferencePhases {
    pub(in super::super) fn record_counts(&self, facts: &mut ValueFlowFacts) {
        facts.reference_phase_indexes += 1;
        facts.reference_phase_nodes += self.stats.nodes;
        facts.reference_phase_path_tokens += self.stats.path_tokens;
        facts.reference_phase_pending_tokens += self.stats.pending_tokens;
        facts.reference_phase_effects += self.effects.as_ref().map_or(0, BTreeSet::len);
        facts.reference_phase_queries += self.stats.queries.get();
    }
}
