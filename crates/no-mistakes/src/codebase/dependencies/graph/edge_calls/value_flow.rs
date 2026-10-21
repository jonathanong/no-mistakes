use super::*;
use crate::codebase::dependencies::extract::{
    ValueType, ValueBinding, ValueCall, ValueExpression, ValueParameter, ValueProjection,
};

#[path = "value_flow/models.rs"]
mod models;
pub(super) use models::FlowFileIndex;
use models::{ParameterInput, ParameterInputs};
#[cfg(feature = "test-instrumentation")]
#[path = "value_flow/admission_probe.rs"]
mod admission_probe;
#[path = "value_flow/known_callable_admission.rs"]
mod known_callable_admission;
#[cfg(feature = "test-instrumentation")]
#[path = "value_flow/admission_mode.rs"]
pub(super) mod admission_mode;
#[path = "value_flow/types.rs"]
mod types;
#[cfg(feature = "test-instrumentation")]
use super::value_flow_metrics::{
    ReplacementIncomingProbe, ReplacementLookupProbe, ReplacementPhysicalProbe,
    ReplacementProbeSnapshot, ReplacementRhsProbe, ReplacementSourceCallProbe,
};
use types::{BoundArguments, FlowValue};

/// Resolves returned values and receiver parameters over parser-owned facts and
/// the same prepared callable indexes as ordinary calls. Incoming argument
/// bindings must agree; an unknown or unrelated argument cannot manufacture a
/// shared effect edge for every invocation of a helper.
struct ValueFlowResolution<'a, 'b> {
    inputs: &'a GraphEdgeBuildInputs<'b>,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    indexes: &'a CallableResolutionIndexes,
    incoming: &'a ParameterInputs,
    supers: &'a ConstructorRoutes,
    replacements: Option<&'a replacement_catalog::ReplacementCatalog>,
    replacement_read: bool,
    foreign_acquisition: bool,
    used_parameters: FxHashSet<(std::path::PathBuf, CallableId)>,
    parameters_enabled: bool,
    initializing: Vec<(std::path::PathBuf, CallableId, usize)>,
    cache: &'a ParameterCache,
    visiting: FxHashSet<(std::path::PathBuf, usize, String)>,
    calls_visiting: FxHashSet<(std::path::PathBuf, u32, u32)>,
    // Immutable object/array fields retain identity throughout one projection.
    fields_visiting: FxHashSet<usize>,
    default_origins: default_origins::Origins,
    acquisition_context: default_origins::Origins,
    invocation_read: Option<default_origins::InvocationRead>,
    // Transient argument-source bound from the existing call facts. The read's
    // file/end/owner identity already identifies this span in parameter keys.
    invocation_start: Option<u32>,
    executing_read: Option<(
        &'a std::path::Path,
        CallableId,
        execution_quality::ExecutionPhase,
    )>,
    module_writer_phases: &'a execution_phases::ModuleWriterPhases,
    writer_dependencies: std::cell::RefCell<FxHashSet<(std::path::PathBuf, CallableId)>>,
    module_snapshot: Option<(std::path::PathBuf, u32)>,
    // Only the current copied-member initializer owns this acquisition phase.
    member_initializer: Option<(&'a std::path::Path, u32, Option<CallableId>, bool)>,
    member_read: Option<(std::path::PathBuf, u32, Option<CallableId>)>,
}

#[path = "value_flow/execution_quality.rs"]
mod execution_quality;
#[path = "value_flow/execution_reads.rs"]
mod execution_reads;
#[path = "value_flow/member_projection.rs"]
mod member_projection;
#[path = "value_flow/member_reads.rs"]
mod member_reads;
#[path = "value_flow/named.rs"]
mod named;
#[path = "value_flow/named_imports.rs"]
mod named_imports;
#[path = "value_flow/refine.rs"]
mod refine;
#[path = "value_flow/returned.rs"]
mod returned;
#[path = "value_flow/values.rs"]
mod values;
pub(super) use refine::refine_value_calls;

#[path = "value_flow/factory_bindings.rs"]
mod factory_bindings;

#[path = "value_flow/exports.rs"]
mod exports;
pub(super) use exports::resolve_value_exports;

#[path = "value_flow/inline_inputs.rs"]
mod inline_inputs;

#[path = "value_flow/parameters.rs"]
mod parameters;

pub(super) type ConstructorRoutes = FxHashMap<std::path::PathBuf, FxHashMap<CallableId, NodeId>>;
#[derive(Clone, Copy)]
struct ValueInputs<'a> {
    module_writer_phases: &'a execution_phases::ModuleWriterPhases,
    cache: &'a ParameterCache,
    incoming: &'a ParameterInputs,
    supers: &'a ConstructorRoutes,
    replacements: Option<&'a replacement_catalog::ReplacementCatalog>,
}
#[path = "value_flow/constructors.rs"]
mod constructors;
pub(super) use constructors::{collect_constructors, prepare_constructor_routes};

#[path = "value_flow/parameter_cache.rs"]
mod parameter_cache;
pub(super) use parameter_cache::ParameterCache;
use parameter_cache::{CacheAccess, CachedParameter, ParameterKey};

#[path = "value_flow/default_origins.rs"]
mod default_origins;
use default_origins::{OriginKind, OriginNode};

#[path = "value_flow/origin_tree.rs"]
mod origin_tree;
#[path = "value_flow/parameter_projection.rs"]
mod parameter_projection;

#[path = "value_flow/receiver_origins.rs"]
mod receiver_origins;

#[path = "value_flow/origin_sources.rs"]
mod origin_sources;

#[path = "value_flow/receiver_contexts.rs"]
mod receiver_contexts;

#[path = "value_flow/origin_walk.rs"]
mod origin_walk;
#[path = "value_flow/parameter_reads.rs"]
mod parameter_reads;
#[path = "value_flow/prepared_receivers.rs"]
mod prepared_receivers;
#[path = "value_flow/receiver_reads.rs"]
mod receiver_reads;

#[path = "value_flow/receiver_owned_contexts.rs"]
mod receiver_owned_contexts;

#[path = "value_flow/agreement.rs"]
mod agreement;
#[path = "value_flow/file_index.rs"]
mod file_index;
#[path = "value_flow/provenance.rs"]
mod provenance;
use agreement::agreed_value;

#[path = "value_flow/invocation_argument.rs"]
mod invocation_argument;

#[path = "value_flow/replacement_catalog.rs"]
mod replacement_catalog;

#[path = "value_flow/replacement_values.rs"]
mod replacement_values;

#[path = "value_flow/binding_reads.rs"]
mod binding_reads;

#[path = "value_flow/execution_phases.rs"]
mod execution_phases;
#[path = "value_flow/parameter_repeat.rs"]
mod parameter_repeat;

#[cfg(test)]
#[path = "value_flow/execution_layout_tests.rs"]
mod execution_layout_tests;

#[path = "value_flow/apply_value.rs"]
mod apply_value;

#[path = "value_flow/parameter_values.rs"]
mod parameter_values;

#[path = "value_flow/annotation_values.rs"]
mod annotation_values;

#[path = "value_flow/choice_targets.rs"]
mod choice_targets;

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "value_flow/site_storage_tests.rs"]
pub(super) mod site_storage;

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "value_flow/opaque_entry_mode.rs"]
pub(super) mod opaque_entry_mode;
