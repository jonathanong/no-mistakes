use super::*;

pub(super) type DefaultOriginIndex =
    FxHashMap<usize, FxHashMap<String, Vec<default_origins::ReceiverOrigin>>>;
// None is a dynamic segment; Some("*") is an ordinary literal property.
pub type ValueMutationPath = Vec<Option<String>>;
#[path = "extract_value_flow/receiver_histories.rs"]
mod receiver_histories;
pub(super) type PriorMemberMutations = receiver_histories::ReceiverHistories;
pub type ValueReceiverMutations =
    FxHashMap<usize, FxHashMap<String, std::sync::Arc<mutation_history::ValueMutationHistory>>>;
pub type ValueDefaultMutations = FxHashMap<usize, Vec<ValueDefaultMutationSource>>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueDefaultMutationSource {
    pub path: ValueMutationPath,
    pub history: std::sync::Arc<mutation_history::ValueMutationHistory>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueMutationInfo {
    pub offset: u32,
    pub contexts: FxHashSet<Option<CallableId>>,
}
pub(super) type ReferenceLinks = FxHashMap<usize, FxHashMap<String, Vec<(usize, String)>>>;

/// Syntactic value flow retained by the import collector. Resolution uses the
/// prepared catalog and never reparses a factory or helper body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueFlowFacts {
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
    pub bindings: Vec<ValueBinding>,
    pub parameters: Vec<ValueParameter>,
    pub returns: Vec<(CallableId, ValueExpression)>,
    pub calls: Vec<ValueCall>,
    pub type_exports: Vec<ExportedBinding>,
    pub async_callables: FxHashSet<CallableId>,
    pub generator_callables: FxHashSet<CallableId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpression {
    Named {
        name: String,
        scope: Option<usize>,
        offset: u32,
        context: Option<CallableId>,
    },
    Call(u32, u32),
    Awaited(std::sync::Arc<ValueExpression>),
    Callable(CallableId),
    Super(CallableId),
    Member(std::sync::Arc<ValueExpression>, String),
    Object(std::sync::Arc<FxHashMap<String, ValueExpression>>, bool),
    Array(std::sync::Arc<FxHashMap<String, ValueExpression>>),
    Undefined,
    Default(
        std::sync::Arc<ValueExpression>,
        std::sync::Arc<ValueExpression>,
    ),
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueBinding {
    pub name: String,
    pub scope: usize,
    pub declared_at: u32,
    pub invalidated_at: Option<u32>,
    pub mutation_contexts: FxHashSet<Option<CallableId>>,
    pub value: ValueExpression,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueProjection {
    Member(String),
    Element(String),
    Default(ValueExpression, usize),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueParameter {
    pub owner: CallableId,
    pub index: usize,
    pub name: String,
    pub scope: usize,
    pub default: ValueExpression,
    pub projection: Vec<ValueProjection>,
    pub initialization_order: usize,
    pub default_order: usize,
    pub default_mutations: Option<std::sync::Arc<ValueDefaultMutations>>,
    pub invalidated_at: Option<u32>,
    pub mutation_contexts: FxHashSet<Option<CallableId>>,
    /// An explicit reference in the annotation namespace, distinct from runtime names.
    pub type_reference: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueCall {
    pub offset: u32,
    pub end: u32,
    pub source_callee: String,
    pub context: Option<CallableId>,
    pub callee: ValueExpression,
    pub arguments: std::sync::Arc<Vec<ValueExpression>>,
    pub bound: bool,
    pub spread_at: Option<usize>,
}
#[path = "extract_value_flow/collect.rs"]
mod collect;

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
