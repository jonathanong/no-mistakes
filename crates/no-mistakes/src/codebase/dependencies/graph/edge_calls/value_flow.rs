use super::*;
use crate::codebase::dependencies::extract::{
    ValueBinding, ValueCall, ValueExpression, ValueParameter,
};

#[derive(Clone, Default)]
pub(super) struct FlowFileIndex {
    bindings: ScopedNameMap<ValueBinding>,
    parameters: ScopedNameMap<ValueParameter>,
    returns: FxHashMap<CallableId, Vec<ValueExpression>>,
    calls: FxHashMap<u32, ValueCall>,
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
        Self {
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
                .map(|call| (call.offset, call.clone()))
                .collect(),
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
enum FlowValue {
    Target(ResolvedCallTarget),
    Returned {
        factory: (String, String),
        executor: Box<FlowValue>,
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
    incoming: FxHashMap<(std::path::PathBuf, CallableId), Vec<(std::path::PathBuf, u32)>>,
    parameters: FxHashMap<(std::path::PathBuf, usize, String), FlowValue>,
    visiting: FxHashSet<(std::path::PathBuf, usize, String)>,
    calls_visiting: FxHashSet<(std::path::PathBuf, u32)>,
}

#[path = "value_flow/values.rs"]
mod values;
#[path = "value_flow/named.rs"]
mod named;
#[path = "value_flow/returned.rs"]
mod returned;
#[path = "value_flow/refine.rs"]
mod refine;
pub(super) use refine::refine_value_calls;

use returned::agreed_value;
