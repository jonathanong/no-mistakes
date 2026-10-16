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

pub(super) type ReferenceLinks = FxHashMap<usize, FxHashMap<String, Vec<(usize, String)>>>;

#[path = "extract_value_flow/value_facts.rs"]
mod value_facts;
pub use value_facts::{ValueExpression, ValueFlowFacts};
#[path = "extract_value_flow/value_descriptors.rs"]
mod value_descriptors;
pub use value_descriptors::{ValueBinding, ValueCall, ValueParameter, ValueProjection};
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
