use super::*;

impl ImportCollector {
    pub(in super::super) fn record_callable_declaration_bindings(
        &mut self,
        declaration: &VariableDeclaration<'_>,
    ) {
        let binding_scope = if declaration.kind == VariableDeclarationKind::Var {
            self.var_scope_stack.last().copied().unwrap_or(0)
        } else {
            self.local_stack.len() - 1
        };
        let binding_scope = self.lexical_scope_ids[binding_scope];
        for declarator in &declaration.declarations {
            let callable_id = match declarator.init.as_ref() {
                Some(Expression::ArrowFunctionExpression(arrow)) => CallableId(arrow.span.start),
                Some(Expression::FunctionExpression(function)) => CallableId(function.span.start),
                Some(Expression::ObjectExpression(_)) => CallableId(declarator.span.start),
                _ => continue,
            };
            if let Some(name) = binding_identifier_name(&declarator.id) {
                self.insert_callable_binding_name_at(binding_scope, name.to_string());
                self.insert_callable_binding_at(binding_scope, name.to_string(), callable_id);
                self.insert_binding_declared_at(
                    binding_scope,
                    name.to_string(),
                    declarator.span.start,
                );
            }
        }
    }
}
