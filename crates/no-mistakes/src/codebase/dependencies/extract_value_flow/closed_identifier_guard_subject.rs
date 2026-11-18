use super::*;

impl ImportCollector {
    pub(super) fn push_call_guard(
        &mut self,
        call: &oxc_ast::ast::CallExpression<'_>,
        argument: usize,
        from: u32,
        until: u32,
        kind: GuardKind,
    ) {
        let some = matches!(kind, GuardKind::ClosedSet { .. })
            .then(|| reference::some_subject(call))
            .flatten();
        let value = if let Some((name, offset)) = some {
            ValueExpression::Named {
                scope: self.callee_binding_scope(&name),
                name,
                offset,
                context: self.current_function_id(),
            }
        } else {
            let Some(value) = self
                .finite_guard_call_indexes
                .get(&call.span.start)
                .and_then(|index| self.value_flow.calls.get(*index))
                .and_then(|value| value.arguments.get(argument))
            else {
                return;
            };
            value.clone()
        };
        let Some((name, scope, members)) = reference::value(&value) else {
            return;
        };
        if self.collect_identifier_provenance
            && self.value_flow.identifier_writes.iter().any(|write| {
                write.scope == scope
                    && write.name == name
                    && reference::write_overlaps(&write.members, &members)
                    && (write.context != self.current_function_id()
                        || write.at >= call.span.start && write.at < from)
            })
        {
            return;
        }
        let subject = self.guard_subject(&name, scope, from, &members);
        self.value_flow
            .finite_identifier_guards
            .push(FiniteIdentifierGuard {
                context: self.current_function_id(),
                name,
                members,
                from,
                until,
                kind,
                subject,
            });
    }

    fn guard_subject(
        &self,
        name: &str,
        scope: usize,
        at: u32,
        members: &[String],
    ) -> Option<GuardSubject> {
        if let Some(binding) = self.value_flow.bindings.iter().rev().find(|binding| {
            binding.scope == scope
                && binding.name == name
                && binding.declared_at <= at
                && binding.invalidated_at.is_none_or(|end| at < end)
        }) {
            if !members.is_empty() && matches!(binding.value, ValueExpression::Unknown) {
                return None;
            }
            return Some(GuardSubject::Binding {
                scope,
                declared_at: binding.declared_at,
            });
        }
        self.value_flow.parameters.iter().find_map(|parameter| {
            (parameter.scope == scope && parameter.name == name && parameter.projection.is_empty())
                .then_some(GuardSubject::Parameter {
                    scope,
                    owner: parameter.owner,
                    index: parameter.index,
                })
        })
    }
}
