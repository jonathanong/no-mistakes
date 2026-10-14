use super::*;
use crate::codebase::dependencies::extract::{
    ValueBinding, ValueCall, ValueExpression, ValueParameter, ValueProjection,
};

#[derive(Clone, Default)]
pub(super) struct FlowFileIndex {
    synthetic_callbacks: FxHashSet<String>,
    immediate_factory_results: FxHashSet<(u32, u32)>,
    receiver_mutations: Option<std::sync::Arc<crate::codebase::dependencies::extract::ValueReceiverMutations>>,
    bindings: ScopedNameMap<ValueBinding>,
    parameters: ScopedNameMap<ValueParameter>,
    returns: FxHashMap<CallableId, Vec<ValueExpression>>,
    calls: FxHashMap<(u32, u32), ValueCall>,
    sites: FxHashMap<u32, FxHashMap<String, (u32, u32)>>,
    scopes: FxHashMap<CallableId, String>,
    async_callables: FxHashSet<CallableId>,
    generator_callables: FxHashSet<CallableId>,
}
impl FlowFileIndex {
    pub(super) fn new(
        flow: &crate::codebase::dependencies::extract::ValueFlowFacts,
        scopes: &[(CallableId, String)],
        function_calls: &[crate::codebase::dependencies::extract::FunctionCall],
    ) -> Self {
        let mut returns: FxHashMap<_, Vec<_>> = fx_map();
        for (owner, value) in &flow.returns {
            returns.entry(*owner).or_default().push(value.clone());
        }
        let mut sites: FxHashMap<_, FxHashMap<_, _>> = fx_map();
        for call in &flow.calls {
            sites
                .entry(call.offset)
                .or_default()
                .insert(call.source_callee.clone(), (call.offset, call.end));
        }
        Self {
            synthetic_callbacks: function_calls.iter().filter(|call|
                call.invocation == InvocationKind::Callback && call.callee_binding_scope.is_none())
                .map(|call| call.callee.clone()).collect(),
            immediate_factory_results: flow.calls.iter().filter_map(|call| inline_inputs::immediate_call(&call.callee)).collect(),
            receiver_mutations: flow.receiver_mutations.clone(),
            async_callables: flow.async_callables.clone(),
            generator_callables: flow.generator_callables.clone(),
            sites,
            scopes: scopes.iter().cloned().collect(),
            bindings: index_scoped_names(
                flow.bindings
                    .iter()
                    .map(|value| ((value.scope, value.name.clone()), value.clone())),
            ),
            parameters: index_scoped_names(
                flow.parameters
                    .iter()
                    .map(|value| ((value.scope, value.name.clone()), value.clone())),
            ),
            returns,
            calls: flow
                .calls
                .iter()
                .map(|call| ((call.offset, call.end), call.clone()))
                .collect(),
        }
    }
}
#[path = "value_flow/types.rs"]
mod types;
use types::{FlowValue, BoundArguments};

type ParameterInputs =
    FxHashMap<(std::path::PathBuf, CallableId), Vec<ParameterInput>>;

#[derive(Clone, PartialEq, Eq)]
struct ParameterInput {
    file: std::path::PathBuf,
    offset: u32,
    callee: String,
    synthetic_callback: bool,
    callback: bool,
}
impl ParameterInput {
    fn from_site(site: &ResolvedCallSite, indexes: &CallableResolutionIndexes, facts: &dyn TsFactLookup) -> Self {
        Self { callback: site.invocation == InvocationKind::Callback, file: site.file.clone(), offset: site.offset, callee: site.source_callee.clone(),
            synthetic_callback: site.invocation == InvocationKind::Callback && indexes.file(facts, &site.file)
                .is_some_and(|index| index.values.synthetic_callbacks.contains(site.source_callee.as_str())) }
    }
}

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
    used_parameters: FxHashSet<(std::path::PathBuf, CallableId)>,
    parameters_enabled: bool,
    initializing: Vec<(std::path::PathBuf, CallableId, usize)>,
    cache: &'a ParameterCache,
    visiting: FxHashSet<(std::path::PathBuf, usize, String)>,
    calls_visiting: FxHashSet<(std::path::PathBuf, u32, u32)>,
    default_origins: default_origins::Origins,
    invocation_read: Option<(std::path::PathBuf, u32, Option<CallableId>, CallableId)>,
}

#[path = "value_flow/named.rs"]
mod named;
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
    cache: &'a ParameterCache,
    incoming: &'a ParameterInputs,
    supers: &'a ConstructorRoutes,
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

#[path = "value_flow/provenance.rs"]
mod provenance;
#[path = "value_flow/agreement.rs"]
mod agreement;
use agreement::agreed_value;
