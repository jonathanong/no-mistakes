use super::*;

impl FlowFileIndex {
    pub(in super::super) fn new(
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
            callable_value_uses: flow.callable_value_uses.clone(),
            synthetic_callbacks: function_calls
                .iter()
                .filter(|call| {
                    call.invocation == InvocationKind::Callback
                        && call.callee_binding_scope.is_none()
                })
                .map(|call| call.callee.clone())
                .collect(),
            immediate_factory_results: flow
                .calls
                .iter()
                .filter_map(|call| inline_inputs::immediate_call(&call.callee))
                .collect(),
            receiver_mutations: flow.receiver_mutations.as_ref().map(|histories| {
                std::sync::Arc::new(prepared_receivers::PreparedReceiverMutations::new(
                    std::sync::Arc::clone(histories),
                ))
            }),
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
