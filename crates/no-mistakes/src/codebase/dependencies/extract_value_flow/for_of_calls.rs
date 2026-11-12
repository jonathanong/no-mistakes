use super::*;
#[path = "for_of_call_expressions.rs"]
mod expressions;

pub(super) fn collect(body: &Statement<'_>, name: &str, source: Option<&str>) -> Option<Vec<u32>> {
    let mut calls = Vec::new();
    statements(body, name, source, true, &mut 128, &mut calls).then_some(calls)
}

fn statements(
    body: &Statement<'_>,
    name: &str,
    source: Option<&str>,
    project: bool,
    budget: &mut usize,
    calls: &mut Vec<u32>,
) -> bool {
    if *budget == 0 {
        return false;
    }
    *budget -= 1;
    match body {
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .all(|statement| statements(statement, name, source, project, budget, calls)),
        Statement::ExpressionStatement(statement) => {
            expressions::expression(&statement.expression, name, source, project, calls)
        }
        Statement::VariableDeclaration(declaration) => {
            declaration.kind == VariableDeclarationKind::Const
                && declaration.declarations.iter().all(|declarator| {
                    matches!(&declarator.id, BindingPattern::BindingIdentifier(identifier)
                    if identifier.name != name && Some(identifier.name.as_str()) != source)
                        && declarator.init.as_ref().is_some_and(|expression| {
                            expressions::expression(expression, name, source, project, calls)
                        })
                })
        }
        Statement::TryStatement(statement) => {
            statement.finalizer.is_none()
                && statement
                    .block
                    .body
                    .iter()
                    .all(|statement| statements(statement, name, source, project, budget, calls))
                && statement.handler.as_ref().is_none_or(|handler| {
                    handler.param.as_ref().is_none_or(|parameter| {
                        !binding_names(&parameter.pattern)
                            .iter()
                            .any(|bound| bound == name || Some(bound.as_str()) == source)
                    }) && handler
                        .body
                        .body
                        .iter()
                        .all(|statement| statements(statement, name, source, false, budget, calls))
                })
        }
        _ => false,
    }
}

impl ImportCollector {
    pub(in super::super::super) fn record_for_of_reference(
        &mut self,
        identifier: &IdentifierReference<'_>,
    ) {
        if !self.collect_value_flow
            || self
                .receiver_escapes
                .for_of_reads
                .contains(&identifier.span.start)
        {
            return;
        }
        self.record_for_of_escape(identifier.name.as_str());
    }
    pub(in super::super::super) fn record_for_of_escape(&mut self, name: &str) {
        if !self.collect_value_flow {
            return;
        }
        if let Some(scope) = self.callee_binding_scope(name).or_else(|| {
            self.predeclared_imported_bindings
                .contains(name)
                .then_some(0)
        }) {
            let owner = self.current_function_id();
            self.receiver_escapes
                .for_of_escapes
                .entry(scope)
                .or_default()
                .entry(name.to_string())
                .or_default()
                .insert(owner);
        }
    }
    pub(in super::super::super) fn for_of_source_closed(&self, source: &Expression<'_>) -> bool {
        if !self.collect_value_flow {
            return false;
        }
        let Expression::Identifier(identifier) = source.get_inner_expression() else {
            return matches!(
                source.get_inner_expression(),
                Expression::ArrayExpression(_)
            );
        };
        self.callee_binding_scope(identifier.name.as_str())
            .or_else(|| {
                self.predeclared_imported_bindings
                    .contains(identifier.name.as_str())
                    .then_some(0)
            })
            .is_none_or(|scope| {
                !self
                    .receiver_escapes
                    .for_of_escapes
                    .get(&scope)
                    .is_some_and(|names| names.contains_key(identifier.name.as_str()))
            })
    }
    pub(in super::super::super) fn finish_for_of_sources(&mut self) {
        let mut escaped: ForOfEscapes = fx_map();
        for (_, binding) in &self.value_flow.for_of.calls {
            let ValueExpression::Named {
                name,
                scope: Some(scope),
                ..
            } = &binding.value
            else {
                continue;
            };
            if let Some(contexts) = self
                .receiver_escapes
                .for_of_escapes
                .get(scope)
                .and_then(|names| names.get(name))
            {
                escaped
                    .entry(*scope)
                    .or_default()
                    .entry(name.clone())
                    .or_insert_with(|| contexts.clone());
            }
        }
        for binding in self.value_flow.bindings.iter().filter(|binding| {
            binding.immutable && matches!(binding.value, ValueExpression::Array(_))
        }) {
            if let Some(contexts) = self
                .receiver_escapes
                .for_of_escapes
                .get(&binding.scope)
                .and_then(|names| names.get(&binding.name))
            {
                escaped
                    .entry(binding.scope)
                    .or_default()
                    .entry(binding.name.clone())
                    .or_insert_with(|| contexts.clone());
            }
        }
        for binding in &self.call_import_bindings {
            if let Some(contexts) = self
                .receiver_escapes
                .for_of_escapes
                .get(&0)
                .and_then(|names| names.get(&binding.local))
            {
                escaped
                    .entry(0)
                    .or_default()
                    .entry(binding.local.clone())
                    .or_insert_with(|| contexts.clone());
            }
        }
        self.value_flow.for_of.escaped =
            (!escaped.is_empty()).then(|| std::sync::Arc::new(escaped));
    }
}
