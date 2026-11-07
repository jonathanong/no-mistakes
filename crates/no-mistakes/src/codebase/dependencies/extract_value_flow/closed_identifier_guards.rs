use super::super::super::simple_callee_name;
use super::plain_call;
use super::*;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_span::GetSpan;

#[path = "closed_identifier_guard_reference.rs"]
mod reference;

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
        reference::closed_set_guards(statement, |call| {
            let Expression::StaticMemberExpression(member) =
                crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
            else {
                return;
            };
            if let Some(collection) = reference::identifier_name(&member.object) {
                let method = if member.property.name == "has" {
                    SetMethod::Has
                } else {
                    SetMethod::Includes
                };
                self.push_call_guard(
                    call,
                    0,
                    statement.span().end,
                    until,
                    GuardKind::ClosedSet { collection, method },
                );
            }
        });
        self.note_assertion_guard(statement, until);
        super::metadata::note_guard(self, statement, until);
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
        match statement {
            Statement::ExpressionStatement(expression) => {
                if let Some(call) = plain_call(&expression.expression) {
                    self.note_assertion_call(call, statement.span().end, until);
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for binding in &declaration.declarations {
                    if let Some(call) = binding.init.as_ref().and_then(|value| plain_call(value)) {
                        self.note_assertion_call(call, binding.span.end, until);
                    }
                }
            }
            _ => {}
        }
    }

    fn note_assertion_call(
        &mut self,
        call: &oxc_ast::ast::CallExpression<'_>,
        from: u32,
        until: u32,
    ) {
        if call
            .arguments
            .iter()
            .any(|argument| matches!(argument, Argument::SpreadElement(_)))
        {
            return;
        }
        let Some(callee) = simple_callee_name(&call.callee) else {
            return;
        };
        for argument in 0..call.arguments.len() {
            if call.arguments[argument]
                .as_expression()
                .and_then(reference::expression)
                .is_none()
            {
                continue;
            }
            self.push_call_guard(
                call,
                argument,
                from,
                until,
                GuardKind::Assertion {
                    callee: callee.clone(),
                    argument,
                    call: call.span.start,
                },
            );
        }
    }

    fn push_call_guard(
        &mut self,
        call: &oxc_ast::ast::CallExpression<'_>,
        argument: usize,
        from: u32,
        until: u32,
        kind: GuardKind,
    ) {
        let Some(value) = self
            .finite_guard_call_indexes
            .get(&call.span.start)
            .and_then(|index| self.value_flow.calls.get(*index))
            .and_then(|value| value.arguments.get(argument))
        else {
            return;
        };
        let Some((name, scope, members)) = reference::value(value) else {
            return;
        };
        if self.value_flow.identifier_writes.iter().any(|write| {
            write.scope == scope
                && write.name == name
                && reference::write_overlaps(&write.members, &members)
                && write.at < from
                && (write.at >= call.span.start || write.context != self.current_function_id())
        }) {
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
