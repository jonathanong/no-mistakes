use super::*;
use crate::codebase::dependencies::extract::{
    ValueBinding, ValueCall, ValueExpression, ValueParameter,
};

#[derive(Clone, Default)]
pub(super) struct FlowFileIndex {
    bindings: ScopedNameMap<ValueBinding>,
    parameters: ScopedNameMap<ValueParameter>,
    returns: FxHashMap<CallableId, Vec<ValueExpression>>,
    calls: FxHashMap<(u32, u32), ValueCall>,
    sites: FxHashMap<u32, FxHashMap<String, (u32, u32)>>,
    scopes: FxHashMap<CallableId, String>,
}
impl FlowFileIndex {
    pub(super) fn new(
        flow: &crate::codebase::dependencies::extract::ValueFlowFacts,
        scopes: &[(CallableId, String)],
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
#[derive(Clone, PartialEq, Eq)]
enum FlowValue {
    Target(ResolvedCallTarget),
    Returned {
        factory: Vec<(String, String)>,
        executor: Box<FlowValue>,
        receiver: Option<ResolvedCallTarget>,
    },
    Callable {
        file: std::path::PathBuf,
        scope: String,
        id: CallableId,
    },
    Object(Vec<(String, FlowValue)>),
    Unknown,
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
    incoming: FxHashMap<(std::path::PathBuf, CallableId), Vec<(std::path::PathBuf, u32, String)>>,
    parameters_enabled: bool,
    parameters: FxHashMap<(std::path::PathBuf, usize, String), FlowValue>,
    visiting: FxHashSet<(std::path::PathBuf, usize, String)>,
    calls_visiting: FxHashSet<(std::path::PathBuf, u32, u32)>,
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

use returned::agreed_value;

impl FlowFileIndex {
    pub(super) fn factory_binding(&self, scope: usize, name: &str) -> bool {
        self.factory_value(
            &ValueExpression::Named {
                name: name.to_string(),
                scope: Some(scope),
                offset: u32::MAX,
            },
            &mut fx_set(),
        )
    }
    fn factory_value(
        &self,
        value: &ValueExpression,
        visiting: &mut FxHashSet<(usize, String)>,
    ) -> bool {
        match value {
            ValueExpression::Call(_, _) => true,
            ValueExpression::Member(value, _) => self.factory_value(value, visiting),
            ValueExpression::Named {
                name,
                scope: Some(scope),
                ..
            } => {
                let base = name.split_once('.').map_or(name.as_str(), |(base, _)| base);
                if !visiting.insert((*scope, base.to_string())) {
                    return false;
                }
                scoped_name(&self.bindings, *scope, base).is_some_and(|binding| {
                    binding.invalidated_at.is_none() && self.factory_value(&binding.value, visiting)
                })
            }
            _ => false,
        }
    }
}
