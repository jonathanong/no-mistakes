use super::*;

pub(super) type DefaultOriginIndex =
    FxHashMap<usize, FxHashMap<String, Vec<default_origins::ReceiverOrigin>>>;
#[path = "extract_value_flow/receiver_histories.rs"]
mod receiver_histories;
pub(super) type PriorMemberMutations = receiver_histories::ReceiverHistories;
#[path = "extract_value_flow/mutation_sources.rs"]
mod mutation_sources;
pub use mutation_sources::{
    ValueDefaultMutationSource, ValueDefaultMutations, ValueMutationInfo, ValueMutationPath,
    ValueReceiverMutations,
};
pub(super) type ReferenceLinks = FxHashMap<usize, FxHashMap<String, Vec<(usize, String)>>>;

/// Syntactic value flow retained by the import collector. Resolution uses the
/// prepared catalog and never reparses a factory or helper body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueFlowFacts {
    /// Local callable values can escape through containers, returns, or stores.
    /// Bare call callees are represented by canonical invocation facts instead.
    pub callable_value_uses: Option<std::sync::Arc<FxHashMap<usize, FxHashSet<String>>>>,
    /// Literal callable assignments to a mutable lexical name, including
    /// unknown writes that prevent a closed canonical call edge.
    pub callable_assignments: Vec<CallableAssignment>,
    pub callable_assignment_uses: FxHashSet<(usize, String)>,
    #[cfg(feature = "test-instrumentation")]
    pub reference_link_constructions: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_component_links: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_component_root_steps: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_link_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_path_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_identity_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_capture_nodes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_capture_indexes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_rhs_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_indexes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_nodes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_path_tokens: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_pending_tokens: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_write_entries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_original_entries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_prepared_fresh_entries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_call_indexes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_call_entries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_effects: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_phase_queries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_capture_logical_tokens: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_region_breaks: usize,
    #[cfg(feature = "test-instrumentation")]
    pub early_assignment_phase_writes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub early_assignment_scope_receipts: usize,
    #[cfg(feature = "test-instrumentation")]
    pub early_assignment_catalogs: usize,
    #[cfg(feature = "test-instrumentation")]
    pub early_assignment_segments: usize,
    #[cfg(feature = "test-instrumentation")]
    pub early_assignment_normalization_visits: usize,
    pub receiver_mutations: Option<std::sync::Arc<ValueReceiverMutations>>,
    pub replacements: Option<std::sync::Arc<replacements::ValueReplacementFacts>>,
    pub bindings: Vec<ValueBinding>,
    pub parameters: Vec<ValueParameter>,
    pub returns: Vec<(CallableId, ValueExpression)>,
    pub calls: Vec<ValueCall>,
    pub type_exports: Vec<ExportedBinding>,
    pub type_sources: Option<Box<ValueTypeSources>>,
    pub async_callables: FxHashSet<CallableId>,
    pub generator_callables: FxHashSet<CallableId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableAssignment {
    pub scope: usize,
    pub name: String,
    pub completed_at: u32,
    pub callable: Option<CallableId>,
    pub owner: Option<CallableId>,
    pub region_start: Option<u32>,
    pub guaranteed: bool,
}
#[path = "extract_value_flow/value_descriptors.rs"]
mod value_descriptors;
pub use value_descriptors::{
    ValueBinding, ValueCall, ValueExpression, ValueParameter, ValueProjection,
};
#[path = "extract_value_flow/collect.rs"]
mod collect;
#[path = "extract_value_flow/collect_helpers.rs"]
mod collect_helpers;

#[path = "extract_value_flow/expressions.rs"]
mod expressions;

#[path = "extract_value_flow/adapters.rs"]
mod adapters;
#[path = "extract_value_flow/assigned_members.rs"]
mod assigned_members;
#[path = "extract_value_flow/completion.rs"]
mod completion;
#[path = "extract_value_flow/mutations.rs"]
mod mutations;
#[path = "extract_value_flow/references.rs"]
mod references;

#[path = "extract_value_flow/result_kind.rs"]
mod result_kind;

#[path = "extract_value_flow/default_export.rs"]
mod default_export;
#[path = "extract_value_flow/templates.rs"]
mod templates;

#[path = "extract_value_flow/nested.rs"]
mod nested;

#[path = "extract_value_flow/parameter_patterns.rs"]
mod parameter_patterns;

#[path = "extract_value_flow/default_origins.rs"]
mod default_origins;

#[path = "extract_value_flow/mutation_history.rs"]
mod mutation_history;
pub use mutation_history::ValueMutationHistory;

#[path = "extract_value_flow/write_targets.rs"]
pub(super) mod write_targets;

#[path = "extract_value_flow/object_values.rs"]
mod object_values;

#[path = "extract_value_flow/destructuring_order.rs"]
mod destructuring_order;
#[path = "extract_value_flow/mutation_assignments.rs"]
mod mutation_assignments;

#[path = "extract_value_flow/replacements.rs"]
mod replacements;
pub use replacements::{
    ValueRepeatCall, ValueRepeatRegion, ValueReplacementFacts, ValueReplacementWrite,
};
#[path = "extract_value_flow/reference_replacements.rs"]
mod reference_replacements;

#[path = "extract_value_flow/phase_catalog.rs"]
mod phase_catalog;
#[path = "extract_value_flow/phase_receipts.rs"]
pub(super) mod phase_receipts;
#[path = "extract_value_flow/phase_runner.rs"]
pub(super) mod phase_runner;
#[path = "extract_value_flow/phase_targets.rs"]
mod phase_targets;
#[path = "extract_value_flow/phase_walk.rs"]
mod phase_walk;
#[path = "extract_value_flow/receiver_traversal.rs"]
mod receiver_traversal;

#[path = "extract_value_flow/fresh_receiver_effects.rs"]
pub(super) mod fresh_receiver_effects;
#[path = "extract_value_flow/identity_metadata.rs"]
mod identity_metadata;
#[path = "extract_value_flow/reference_identity.rs"]
mod reference_identity;

#[path = "extract_value_flow/capture_reads.rs"]
mod capture_reads;

#[cfg(test)]
#[path = "extract_value_flow/post_capture_call_tests.rs"]
mod post_capture_call_tests;
#[cfg(test)]
#[path = "extract_value_flow/receiver_identity_tests.rs"]
mod receiver_identity_tests;

#[path = "extract_value_flow/active_reference_phases.rs"]
mod active_reference_phases;

#[path = "extract_value_flow/inert_receiver_rhs.rs"]
mod inert_receiver_rhs;

#[cfg(test)]
#[path = "extract_value_flow/suspension_tests.rs"]
mod suspension_tests;

#[path = "extract_value_flow/callable_uses.rs"]
mod callable_uses;

#[path = "extract_value_flow/type_sources.rs"]
mod type_sources;
pub use type_sources::{ValueTypeSource, ValueTypeSources};
