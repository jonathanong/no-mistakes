use super::*;

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
            let expression = statement.expression.get_inner_expression();
            let expression = match expression {
                Expression::AwaitExpression(value) => value.argument.get_inner_expression(),
                value => value,
            };
            let Expression::CallExpression(call) = expression else {
                return false;
            };
            let Expression::StaticMemberExpression(member) = &call.callee else {
                return false;
            };
            let Expression::Identifier(receiver) = &member.object else {
                return false;
            };
            if !project && matches!(member.property.name.as_str(), "exit" | "abort") {
                return false;
            }
            if Some(receiver.name.as_str()) == source {
                return false;
            }
            if receiver.name == name {
                if !safe_arguments(call, name, source) {
                    return false;
                }
                if project {
                    calls.push(call.span.start);
                }
                true
            } else {
                if project {
                    return false;
                }
                safe_arguments(call, name, source)
            }
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

fn safe_arguments(call: &CallExpression<'_>, name: &str, source: Option<&str>) -> bool {
    call.arguments.iter().all(|argument| {
        match argument
            .as_expression()
            .map(Expression::get_inner_expression)
        {
            Some(Expression::Identifier(value)) => {
                value.name != name && Some(value.name.as_str()) != source
            }
            Some(
                Expression::StringLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::NullLiteral(_),
            ) => true,
            _ => false,
        }
    })
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
        if !self.collect_call_eligibility || !self.collect_value_flow {
            return;
        }
        if let Some(scope) = self.callee_binding_scope(name) {
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
        if !self.collect_call_eligibility {
            return false;
        }
        let Expression::Identifier(identifier) = source.get_inner_expression() else {
            return matches!(
                source.get_inner_expression(),
                Expression::ArrayExpression(_)
            );
        };
        self.callee_binding_scope(identifier.name.as_str())
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
        self.value_flow.for_of.escaped =
            (!escaped.is_empty()).then(|| std::sync::Arc::new(escaped));
    }
}
