#[path = "extract_value_flow/assigned_acquisition_roots.rs"]
mod assigned_acquisition_roots;
#[path = "extract_value_flow/assigned_receiver_origins.rs"]
mod assigned_receiver_origins;
#[path = "extract_value_flow/assigned_receiver_proof.rs"]
mod assigned_receiver_proof;
#[path = "extract_value_flow/assignment_member_values.rs"]
mod assignment_member_values;
#[path = "extract_value_flow/assignment_rhs_frame.rs"]
pub(super) mod assignment_rhs_frame;
use super::*;

pub(super) type DefaultOriginIndex =
    FxHashMap<usize, FxHashMap<String, Vec<default_origins::ReceiverOrigin>>>;
#[path = "extract_value_flow/receiver_histories.rs"]
mod receiver_histories;
pub(super) type PriorMemberMutations = receiver_histories::ReceiverHistories;
#[path = "extract_value_flow/mutation_sources.rs"]
mod mutation_sources;
#[path = "extract_value_flow/reference_output.rs"]
mod reference_output;
pub use mutation_sources::{
    ValueDefaultMutationSource, ValueDefaultMutations, ValueMutationInfo, ValueMutationPath,
    ValueReceiverMutations,
};
pub(super) type ReceiverBindingOrigins = mutable_origins::ReceiverBindingOrigins;
pub(super) type ForeignReceiverWrites = mutable_origins::ForeignReceiverWrites;

pub(super) type ReferenceLinks = FxHashMap<usize, FxHashMap<String, Vec<(usize, String)>>>;

include!("extract_value_flow/value_flow_facts.rs");
/// Small syntactic annotation shapes; resolution uses the prepared fact catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Reference(String),
    Member(Box<ValueType>, String),
    Object(FxHashMap<String, ValueType>),
    Unknown,
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

#[path = "extract_value_flow/assignment_root_values.rs"]
mod assignment_root_values;
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
pub(super) mod inert_receiver_rhs;

#[cfg(test)]
#[path = "extract_value_flow/suspension_tests.rs"]
mod suspension_tests;

#[path = "extract_value_flow/callable_uses.rs"]
mod callable_uses;
#[path = "extract_value_flow/mutable_receivers.rs"]
mod mutable_receivers;

#[cfg(test)]
#[path = "extract_value_flow/mutable_receiver_tests.rs"]
mod mutable_receiver_tests;

#[path = "extract_value_flow/mutable_origins.rs"]
mod mutable_origins;

#[path = "extract_value_flow/mutable_patterns.rs"]
mod mutable_patterns;

#[path = "extract_value_flow/root_phase_capture.rs"]
pub(super) mod root_phase_capture;

#[path = "extract_value_flow/mutable_deferred.rs"]
pub(super) mod mutable_deferred;

#[path = "extract_value_flow/mutation_metadata.rs"]
mod mutation_metadata;

#[path = "extract_value_flow/mutation_projection.rs"]
mod mutation_projection;

#[path = "extract_value_flow/mutation_alias_index.rs"]
mod mutation_alias_index;
#[path = "extract_value_flow/mutation_invalidation.rs"]
mod mutation_invalidation;

#[path = "extract_value_flow/phase_class_fields.rs"]
mod phase_class_fields;

#[path = "extract_value_flow/mutation_proof.rs"]
mod mutation_proof;

#[path = "extract_value_flow/own_data_reads.rs"]
mod own_data_reads;

#[cfg(feature = "test-instrumentation")]
#[path = "extract_value_flow/ordinary_demand_metrics.rs"]
pub(super) mod ordinary_demand_metrics;

#[path = "extract_value_flow/mutable_root_single.rs"]
mod mutable_root_single;

#[cfg(feature = "test-instrumentation")]
#[path = "extract_value_flow/root_target_metrics.rs"]
mod root_target_metrics;
#[path = "extract_value_flow/root_targets.rs"]
pub(super) mod root_targets;

#[path = "extract_value_flow/phase_changed_roots.rs"]
mod phase_changed_roots;
#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "extract_value_flow/phase_changed_roots_tests.rs"]
mod phase_changed_roots_tests;
