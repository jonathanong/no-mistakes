use super::*;

/// Syntactic value flow retained by the import collector. Resolution uses the
/// prepared catalog and never reparses a factory or helper body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueFlowFacts {
    /// Local callable values can escape through containers, returns, or stores.
    /// Bare call callees are represented by canonical invocation facts instead.
    pub callable_value_uses: Option<std::sync::Arc<FxHashMap<usize, FxHashSet<String>>>>,
    #[cfg(feature = "test-instrumentation")]
    pub reference_link_constructions: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_origins: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_nodes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_lookups: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_phase_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_component_links: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_component_root_steps: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_link_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_path_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_binding_visits: usize,
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
