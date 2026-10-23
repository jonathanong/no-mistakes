use super::*;

#[derive(Clone, Default)]
pub(in super::super) struct FlowFileIndex {
    pub(super) callable_value_uses: Option<std::sync::Arc<FxHashMap<usize, FxHashSet<String>>>>,
    pub(super) synthetic_callbacks: FxHashSet<String>,
    pub(super) immediate_factory_results: FxHashSet<(u32, u32)>,
    pub(super) receiver_mutations:
        Option<std::sync::Arc<prepared_receivers::PreparedReceiverMutations>>,
    pub(super) possible_receiver_mutations:
        Option<std::sync::Arc<prepared_receivers::PreparedReceiverMutations>>,
    pub(super) bindings: ScopedNameMap<ValueBinding>,
    pub(super) parameters: ScopedNameMap<ValueParameter>,
    pub(super) returns: FxHashMap<CallableId, Vec<ValueExpression>>,
    pub(super) calls: FxHashMap<(u32, u32), ValueCall>,
    pub(super) sites: FxHashMap<u32, FxHashMap<String, (u32, u32)>>,
    pub(super) scopes: FxHashMap<CallableId, String>,
    pub(super) async_callables: FxHashSet<CallableId>,
    pub(super) generator_callables: FxHashSet<CallableId>,
}
pub(super) type ParameterInputs = FxHashMap<(std::path::PathBuf, CallableId), Vec<ParameterInput>>;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ParameterInput {
    pub(super) file: std::path::PathBuf,
    pub(super) offset: u32,
    pub(super) callee: String,
    pub(super) synthetic_callback: bool,
    pub(super) callback: bool,
}
impl ParameterInput {
    pub(super) fn from_site<'facts>(
        site: &ResolvedCallSite,
        indexes: &CallableResolutionIndexes<'facts>,
        facts: &'facts dyn TsFactLookup,
    ) -> Self {
        Self {
            callback: site.invocation == InvocationKind::Callback,
            file: site.file.clone(),
            offset: site.offset,
            callee: site.source_callee.clone(),
            synthetic_callback: site.invocation == InvocationKind::Callback
                && indexes.file(facts, &site.file).is_some_and(|index| {
                    index
                        .values
                        .synthetic_callbacks
                        .contains(site.source_callee.as_str())
                }),
        }
    }
}
