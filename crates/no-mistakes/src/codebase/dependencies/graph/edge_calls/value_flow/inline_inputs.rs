use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn synchronous_callable(&self, path: &std::path::Path, id: CallableId) -> bool {
        self.indexes.file(self.facts, path).is_some_and(|index| {
            !index.values.async_callables.contains(&id)
                && !index.values.generator_callables.contains(&id)
        })
    }
    pub(super) fn safe_inline_callee(
        &mut self,
        path: &std::path::Path,
        value: &ValueExpression,
    ) -> bool {
        match value {
            ValueExpression::Callable(id) => self.synchronous_callable(path, *id),
            ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => {
                self.safe_inline_callee(path, base)
            }
            ValueExpression::Call(offset, end) => {
                let Some(call) = self
                    .indexes
                    .file(self.facts, path)
                    .and_then(|index| index.values.calls.get(&(*offset, *end)).cloned())
                else {
                    return false;
                };
                if call.bound {
                    return match &call.callee {
                        ValueExpression::Named { .. } => true,
                        value => self.safe_inline_callee(path, value),
                    };
                }
                let origins = self.default_origins.take();
                let read = self.invocation_read.take();
                let start = self.invocation_start.take();
                let mut target = self.value(path, &call.callee);
                self.default_origins = origins;
                self.invocation_read = read;
                self.invocation_start = start;
                while let FlowValue::Bound { target: inner, .. } | FlowValue::Possible(inner) =
                    target
                {
                    target = *inner;
                }
                let owner = match target {
                    FlowValue::Callable { file, id, .. } => (file, id),
                    FlowValue::Target(ResolvedCallTarget::ModuleExport {
                        repository_target: Some((file, _)),
                        callable_id: Some(id),
                        ..
                    }) => (file, id),
                    _ => return false,
                };
                self.used_parameters.insert(owner.clone());
                // Use the canonical incoming relationships: another factory result
                // can escape through an alias, import, or opaque call argument.
                self.cache.factory_agrees(owner.clone(), || {
                    let Some(inputs) = self.incoming.get(&owner) else {
                        return false;
                    };
                    let mut authored = false;
                    for input in inputs {
                        if input.synthetic_callback {
                            continue;
                        }
                        if input.callback {
                            return false;
                        }
                        authored = true;
                        let Some(index) = self.indexes.file(self.facts, &input.file) else {
                            return false;
                        };
                        let Some(key) = index
                            .values
                            .sites
                            .get(&input.offset)
                            .and_then(|sites| sites.get(&input.callee))
                        else {
                            return false;
                        };
                        if !index.values.immediate_factory_results.contains(key) {
                            return false;
                        }
                    }
                    authored
                })
            }
            _ => false,
        }
    }
}

pub(super) fn immediate_call(value: &ValueExpression) -> Option<(u32, u32)> {
    match value {
        ValueExpression::Call(offset, end) => Some((*offset, *end)),
        ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => immediate_call(base),
        _ => None,
    }
}

pub(super) fn inline_callee(value: &ValueExpression) -> bool {
    match value {
        ValueExpression::Callable(_) | ValueExpression::Call(_, _) => true,
        ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => inline_callee(base),
        _ => false,
    }
}
