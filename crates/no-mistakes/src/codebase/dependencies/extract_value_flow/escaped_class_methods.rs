use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_instance_class(
        &mut self,
        declarator: &VariableDeclarator<'_>,
    ) {
        if let Some(class_id) = declarator.init.as_ref().and_then(|init| {
            let Expression::NewExpression(new) =
                crate::codebase::ts_source::unwrap_ts_wrappers(init)
            else {
                return None;
            };
            let name = simple_callee_name(&new.callee)?;
            self.callable_binding_at(self.callee_binding_scope(&name).unwrap_or(0), &name)
        }) {
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
        self.note_instance_aliases(declarator);
    }

    pub(super) fn note_class_method(&mut self, call: &CallExpression<'_>) {
        let Some((method, arguments)) = self.resolve_class_method(call) else {
            return;
        };
        self.receiver_escapes
            .methods
            .push((call.span.start, call.span.end, method, arguments));
    }
}
