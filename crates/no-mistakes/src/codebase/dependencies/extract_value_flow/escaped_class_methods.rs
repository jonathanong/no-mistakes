use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_instance_class(
        &mut self,
        declarator: &VariableDeclarator<'_>,
    ) {
        let Some(init) = &declarator.init else {
            return;
        };
        let Expression::NewExpression(new) = crate::codebase::ts_source::unwrap_ts_wrappers(init)
        else {
            return;
        };
        let Some(name) = simple_callee_name(&new.callee) else {
            return;
        };
        let Some(class_id) =
            self.callable_binding_at(self.callee_binding_scope(&name).unwrap_or(0), &name)
        else {
            return;
        };
        for binding in binding_names(&declarator.id) {
            self.receiver_escapes.instances.insert(
                (
                    self.callee_binding_scope(&binding)
                        .unwrap_or(self.current_lexical_scope_id()),
                    binding,
                ),
                class_id,
            );
        }
    }

    pub(super) fn note_class_method(&mut self, call: &CallExpression<'_>) {
        let Expression::StaticMemberExpression(member) =
            crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
        else {
            return;
        };
        let Some(class) = class_id(self, &member.object) else {
            return;
        };
        let Some(method) = owner_member_id(
            &self.aggregate_callable_member_ids,
            class,
            member.property.name.as_str(),
        ) else {
            return;
        };
        let arguments = call
            .arguments
            .iter()
            .map(|argument| {
                argument
                    .as_expression()
                    .map_or(ValueExpression::Unknown, |expression| {
                        self.value_expression(expression)
                    })
            })
            .collect();
        self.receiver_escapes
            .methods
            .push((call.span.start, call.span.end, method, arguments));
    }
}

fn class_id(collector: &ImportCollector, object: &Expression<'_>) -> Option<CallableId> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(object) {
        Expression::NewExpression(new) => simple_callee_name(&new.callee).and_then(|name| {
            collector.callable_binding_at(collector.callee_binding_scope(&name).unwrap_or(0), &name)
        }),
        Expression::Identifier(identifier) => collector
            .receiver_escapes
            .instances
            .get(&(
                collector
                    .callee_binding_scope(identifier.name.as_str())
                    .unwrap_or(0),
                identifier.name.to_string(),
            ))
            .copied(),
        _ => None,
    }
}
