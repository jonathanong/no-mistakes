/// Syntactic value flow retained by the import collector. Resolution uses the
/// prepared catalog and never reparses a factory or helper body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueFlowFacts {
    /// Prepared nullish expressions can add known plain-name fallback effects.
    pub has_nullish_alternatives: bool,
    /// Local callable values can escape through containers, returns, or stores.
    /// Bare call callees are represented by canonical invocation facts instead.
    pub callable_value_uses: Option<std::sync::Arc<FxHashMap<usize, FxHashSet<String>>>>,
    #[cfg(feature = "test-instrumentation")]
    pub replacement_statement_checks: usize,
    #[cfg(feature = "test-instrumentation")]
    pub replacement_expression_checks: usize,
    #[cfg(feature = "test-instrumentation")]
    pub replacement_expression_guard_entries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub replacement_region_frames: usize,
    #[cfg(feature = "test-instrumentation")]
    pub replacement_loop_classifications: usize,
    #[cfg(feature = "test-instrumentation")]
    pub callable_assignment_name_clones: usize,
    #[cfg(feature = "test-instrumentation")]
    pub callable_assignment_target_constructions: usize,
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
    pub type_shapes: FxHashMap<String, ValueType>,
    pub async_callables: FxHashSet<CallableId>,
    pub generator_callables: FxHashSet<CallableId>,
}
