use super::*;

impl ImportCollector {
    pub fn record_finite_guard_call(
        &mut self,
        call: &oxc_ast::ast::CallExpression<'_>,
        index: usize,
    ) {
        let subject = reference::some_subject(call).map(|(name, offset)| ValueExpression::Named {
            scope: self.callee_binding_scope(&name),
            name,
            offset,
            context: self.current_function_id(),
        });
        let collection_scope = match call.callee.get_inner_expression() {
            Expression::StaticMemberExpression(member) => {
                reference::identifier_name(&member.object)
                    .and_then(|name| self.callee_binding_scope(&name))
            }
            _ => None,
        };
        self.finite_guard_calls.insert(
            call.span.start,
            GuardCall {
                index,
                subject,
                collection_scope,
            },
        );
    }

    pub(super) fn push_call_guard(
        &mut self,
        call: &oxc_ast::ast::CallExpression<'_>,
        argument: usize,
        from: u32,
        until: u32,
        kind: GuardKind,
    ) {
        let Some(captured) = self.finite_guard_calls.get(&call.span.start) else {
            return;
        };
        let value = if matches!(kind, GuardKind::ClosedSet { .. }) {
            captured.subject.as_ref()
        } else {
            None
        }
        .or_else(|| {
            self.value_flow
                .calls
                .get(captured.index)?
                .arguments
                .get(argument)
        });
        let Some(value) = value else { return };
        let Some((name, scope, members)) = reference::value(value) else {
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
