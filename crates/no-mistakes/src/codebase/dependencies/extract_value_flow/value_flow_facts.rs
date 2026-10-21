/// Syntactic value flow retained by the import collector. Resolution uses the
/// prepared catalog and never reparses a factory or helper body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueFlowFacts {
    #[cfg(feature = "test-instrumentation")]
    pub module_provider_work: [usize; 3],
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
    pub reference_callable_alias_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_callable_alias_index_builds: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_callable_alias_index_entries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_forward_sources: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_forward_writer_replays: usize,
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
    pub reference_rhs_origin_queries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_rhs_receipt_queries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_rhs_destinations: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_rhs_barriers: usize,
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
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_origins: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_nodes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_lookups: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_snapshot_queries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_snapshot_work: [usize; 2],
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_predecessor_queries: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_insertion_order: [usize; 3],
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_snapshot_tree_buffers: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_membership_probes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_owned_keys: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_visits: usize,
    /// Origin projections that required a heap-backed pending stack.
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_pending_spills: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_only_phase_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_binding_visits: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_root_probes: usize,
    /// Actual calls through the version-aware traversal entry/exit selector.
    #[cfg(feature = "test-instrumentation")]
    pub reference_version_dispatches: usize,
    /// Full receiver-writer metadata captures; lexical writes need no capture.
    #[cfg(feature = "test-instrumentation")]
    pub reference_metadata_captures: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_rebind_calls: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_rebind_name_allocations: usize,
    /// Proven terminal paths in the mutable-coordinate traversal.
    #[cfg(feature = "test-instrumentation")]
    pub reference_terminal_paths: usize,
    /// Actual active/visited bookkeeping attempts, including rejected revisits.
    #[cfg(feature = "test-instrumentation")]
    pub reference_successor_bookkeeping: usize,
    /// Allocated overflow version buckets in mutable receiver traversals.
    #[cfg(feature = "test-instrumentation")]
    pub reference_version_table_capacity: usize,
    /// Actual active/visited hash buffers constructed by mutation traversals.
    #[cfg(feature = "test-instrumentation")]
    pub reference_state_heap_buffers: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_output_heap_buffers: usize,
    /// Actual outer-scope hash buffers retained by per-writer invalidation sets.
    #[cfg(feature = "test-instrumentation")]
    pub reference_affected_scope_buffers: usize,
    #[cfg(feature = "test-instrumentation")]
    pub reference_affected_name_buffers: usize,
    /// Actual stores to earliest binding/parameter/callable invalidation offsets.
    #[cfg(feature = "test-instrumentation")]
    pub reference_invalidation_stores: usize,
    /// Sum of each mutation traversal's peak pending heap capacity, in entries.
    #[cfg(feature = "test-instrumentation")]
    pub reference_pending_capacity: usize,
    /// Sum of those peak heap buffers in bytes, excluding inline stack storage.
    #[cfg(feature = "test-instrumentation")]
    pub reference_pending_bytes: usize,
    #[cfg(feature = "test-instrumentation")]
    pub receiver_root_target_decompositions: usize,
    pub receiver_mutations: Option<std::sync::Arc<ValueReceiverMutations>>,
    /// Newly recovered escape origins retain the original callable as possible.
    /// Historical definite mutation semantics remain in `receiver_mutations`.
    pub possible_receiver_mutations: Option<std::sync::Arc<ValueReceiverMutations>>,
    pub replacements: Option<std::sync::Arc<replacements::ValueReplacementFacts>>,
    pub bindings: Vec<ValueBinding>,
    /// Iterable source for local loop bindings, for provenance only. These are
    /// not runtime element values and never enter ordinary value resolution.
    pub iteration_bindings: Vec<ValueBinding>,
    pub parameters: Vec<ValueParameter>,
    pub returns: Vec<(CallableId, ValueExpression)>,
    pub calls: Vec<ValueCall>,
    pub type_exports: Vec<ExportedBinding>,
    pub type_shapes: FxHashMap<String, ValueType>,
    pub async_callables: FxHashSet<CallableId>,
    pub generator_callables: FxHashSet<CallableId>,
}
