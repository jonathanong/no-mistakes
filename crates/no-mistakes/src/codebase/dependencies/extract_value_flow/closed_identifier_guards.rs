use super::super::super::simple_callee_name;
use super::plain_call;
use super::*;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_span::GetSpan;

#[path = "closed_identifier_guard_reference.rs"]
mod reference;
#[path = "closed_identifier_guard_subject.rs"]
mod subject;

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
                let Some(captured) = self.finite_guard_calls.get(&call.span.start) else {
                    return;
                };
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
                    GuardKind::ClosedSet {
                        collection,
                        method,
                        scope: captured.collection_scope,
                    },
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
}
