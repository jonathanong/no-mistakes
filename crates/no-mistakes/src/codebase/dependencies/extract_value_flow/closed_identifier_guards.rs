use super::super::super::simple_callee_name;
use super::plain_call;
use super::*;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_span::GetSpan;
use oxc_syntax::operator::UnaryOperator;

impl ImportCollector {
    pub(crate) fn record_finite_identifier_guards(
        &mut self,
        statements: &[Statement<'_>],
        until: u32,
    ) {
        if !self.collect_value_flow {
            return;
        }
        // `until` is the end of the block that dominates later statements.
        self.record_identifier_guards(statements, until);
    }

    fn record_identifier_guards(&mut self, statements: &[Statement<'_>], until: u32) {
        for statement in statements {
            self.record_identifier_guard(statement, until);
        }
    }

    fn record_identifier_guard(&mut self, statement: &Statement<'_>, until: u32) {
        if let Some((collection, method, name)) = closed_set_guard(statement) {
            self.push_identifier_guard(
                name,
                statement.span().end,
                until,
                GuardKind::ClosedSet { collection, method },
            );
        }
        self.note_assertion_guard(statement, until);
        match statement {
            Statement::BlockStatement(block) => {
                self.record_identifier_guards(&block.body, block.span.end);
            }
            Statement::IfStatement(branch) => {
                self.record_identifier_guard(&branch.consequent, branch.consequent.span().end);
                if let Some(alternate) = &branch.alternate {
                    self.record_identifier_guard(alternate, alternate.span().end);
                }
            }
            _ => {}
        }
    }

    fn note_assertion_guard(&mut self, statement: &Statement<'_>, until: u32) {
        let Statement::ExpressionStatement(expression) = statement else {
            return;
        };
        let Some(call) = plain_call(&expression.expression) else {
            return;
        };
        if call
            .arguments
            .iter()
            .any(|argument| matches!(argument, Argument::SpreadElement(_)))
        {
            return;
        }
        let Some(callee) = simple_callee_name(&call.callee).filter(|name| !name.contains('.'))
        else {
            return;
        };
        for (argument, value) in call.arguments.iter().enumerate() {
            let Some(name) = value.as_expression().and_then(identifier_name) else {
                continue;
            };
            self.push_identifier_guard(
                name,
                statement.span().end,
                until,
                GuardKind::Assertion {
                    callee: callee.clone(),
                    argument,
                },
            );
        }
    }

    fn push_identifier_guard(&mut self, name: String, from: u32, until: u32, kind: GuardKind) {
        let subject = self.guard_subject(&name, from);
        self.value_flow
            .finite_identifier_guards
            .push(FiniteIdentifierGuard {
                context: self.current_function_id(),
                name,
                from,
                until,
                kind,
                subject,
            });
    }

    fn guard_subject(&self, name: &str, at: u32) -> Option<GuardSubject> {
        let scope = self.callee_binding_scope(name)?;
        if let Some(binding) = self.value_flow.bindings.iter().rev().find(|binding| {
            binding.scope == scope
                && binding.name == name
                && binding.declared_at <= at
                && binding.invalidated_at.is_none_or(|end| at < end)
        }) {
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

fn closed_set_guard(statement: &Statement<'_>) -> Option<(String, SetMethod, String)> {
    let Statement::IfStatement(branch) = statement else {
        return None;
    };
    if branch
        .alternate
        .as_ref()
        .is_some_and(|alternate| super::super::completion::statement_completes(alternate))
        || super::super::completion::statement_completes(&branch.consequent)
    {
        return None;
    }
    let Expression::UnaryExpression(unary) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&branch.test)
    else {
        return None;
    };
    if unary.operator != UnaryOperator::LogicalNot {
        return None;
    }
    let call = plain_call(&unary.argument)?;
    if call.arguments.len() != 1 {
        return None;
    }
    let value = identifier_name(call.arguments[0].as_expression()?)?;
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    let collection = identifier_name(&member.object)?;
    let method = match member.property.name.as_str() {
        "has" => SetMethod::Has,
        "includes" => SetMethod::Includes,
        _ => return None,
    };
    Some((collection, method, value))
}

fn identifier_name(expr: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}
