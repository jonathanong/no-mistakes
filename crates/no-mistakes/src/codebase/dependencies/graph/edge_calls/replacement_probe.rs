use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ReplacementProbeSnapshot {
    pub catalog_builds: usize,
    pub hot_file_indexes: usize,
    pub receiver_normalizations: usize,
    pub canonical_receiver_ids: usize,
    pub local_receiver_ids: usize,
    pub rhs_attempts: usize,
    pub rhs_resolutions: usize,
    pub rhs_known: usize,
    pub retained_write_indices: usize,
    pub region_segments: usize,
    pub normalized_state_payloads: usize,
    pub candidate_leaves: usize,
    pub alternatives_payloads: usize,
    pub alternatives_candidate_leaves: usize,
    /// Actual prepared supported Call/Construct occurrences classified.
    pub call_admission_checks: usize,
    /// Prepared-expression matcher entries, including rejected/depth-limit entries.
    pub call_proof_steps: usize,
    pub opaque_groups: usize,
    pub lookup_count: usize,
    pub prefix_searches: usize,
    /// Calls to the existing allocating qualifier for inherited source frames.
    pub inherited_frame_qualifications: usize,
    pub rhs: Vec<ReplacementRhsProbe>,
    pub incoming: Vec<ReplacementIncomingProbe>,
    pub receiver_initial_admissions_without_target_node: usize,
    pub receiver_initial_admissions_with_target_node: usize,
    pub receiver_original_admission: bool,
    pub receiver_skipped_direct_callable_admissions: usize,
    /// Original order and duplicates retained before canonical edge deduplication.
    pub admission_raw_edges: Vec<String>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct ReplacementRhsProbe {
    pub file: std::path::PathBuf,
    pub write_index: usize,
    pub completed_at: u32,
    pub owner: Option<u32>,
    pub region: Option<usize>,
    pub resolved: bool,
    pub known: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct ReplacementIncomingProbe {
    pub routed_file: std::path::PathBuf,
    pub routed_owner: u32,
    pub caller_file: std::path::PathBuf,
    pub offset: u32,
    pub callee: String,
    pub callback: bool,
    pub synthetic_callback: bool,
    /// Empty means no matching physical site; multiple entries are explicit.
    pub physical: Vec<ReplacementPhysicalProbe>,
    /// None is an explicit missing parser source, never an omitted incoming row.
    pub source_call: Option<ReplacementSourceCallProbe>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct ReplacementPhysicalProbe {
    pub caller_id: Option<u32>,
    pub invocation: String,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct ReplacementSourceCallProbe {
    pub file: std::path::PathBuf,
    pub offset: u32,
    pub end: u32,
    pub context: Option<u32>,
    pub bound: bool,
    pub argument_count: usize,
    pub spread_at: Option<usize>,
    /// Together with this call key, each ordinal identifies its prepared AST
    /// expression. These are syntactic sources, not resolved runtime arguments.
    pub argument_ordinals: Vec<usize>,
}
#[derive(Default)]
pub(in super::super) struct ReplacementLookupProbe {
    count: AtomicUsize,
}
impl ReplacementLookupProbe {
    pub(in super::super) fn record(&self, count: usize) {
        self.count.fetch_add(count, Ordering::Relaxed);
    }
    pub(in super::super) fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}
impl ReplacementProbeSnapshot {
    pub(in super::super) fn target(&mut self, target: &super::super::ResolvedCallTarget) {
        if let super::super::ResolvedCallTarget::Alternatives(alternatives) = target {
            self.normalized_state_payloads += 1;
            self.candidate_leaves += alternatives.known().len();
            if alternatives.known().len() > 1 || alternatives.has_unknown {
                self.alternatives_payloads += 1;
                self.alternatives_candidate_leaves += alternatives.known().len();
            }
        }
    }
    pub(in super::super) fn append(&mut self, other: Self) {
        self.call_admission_checks += other.call_admission_checks;
        self.call_proof_steps += other.call_proof_steps;
        self.opaque_groups += other.opaque_groups;
        self.rhs_attempts += other.rhs_attempts;
        self.rhs_resolutions += other.rhs_resolutions;
        self.rhs_known += other.rhs_known;
        self.normalized_state_payloads += other.normalized_state_payloads;
        self.candidate_leaves += other.candidate_leaves;
        self.alternatives_payloads += other.alternatives_payloads;
        self.alternatives_candidate_leaves += other.alternatives_candidate_leaves;
        self.rhs.extend(other.rhs);
    }
}
thread_local! {
    static LAST_REPLACEMENTS: std::cell::RefCell<ReplacementProbeSnapshot> = std::cell::RefCell::new(ReplacementProbeSnapshot::default());
}
pub(in super::super) fn record_replacements(snapshot: ReplacementProbeSnapshot) {
    LAST_REPLACEMENTS.set(snapshot);
}
pub fn replacement_snapshot() -> ReplacementProbeSnapshot {
    LAST_REPLACEMENTS.with_borrow(Clone::clone)
}
