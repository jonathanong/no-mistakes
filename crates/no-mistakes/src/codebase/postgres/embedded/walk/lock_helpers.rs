use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashMap;
use oxc_ast::ast::{CallExpression, Expression};
mod collect;
mod invocation;
mod summary;
pub(super) use collect::collect;
pub(crate) use summary::{ExecutorAccess, Invocation, Parameter, Summary};

#[derive(Clone)]
pub(super) struct Helper<'a> {
    params: Vec<&'a str>,
    calls: Vec<(usize, ExecutorAccess, &'a CallExpression<'a>)>,
}
pub(super) type LockHelpers<'a> = FxHashMap<String, Helper<'a>>;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn record_lock_helper_call(&mut self, call: &CallExpression<'a>) {
        let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else {
            return;
        };
        let Some(helper) = self.lock_helpers.get(name.name.as_str()).cloned() else {
            self.record_unknown_lock_helper(call, name.name.as_str());
            return;
        };
        if !self.awaited_lock_helper_calls.contains(&call.span.start)
            || helper.calls.iter().any(|(index, expected, _)| {
                call.arguments
                    .get(*index)
                    .and_then(|argument| argument.as_expression())
                    .and_then(summary::access)
                    != Some(*expected)
            })
            || self.lookup(name.name.as_str()).is_some()
            || call.optional
            || call
                .arguments
                .iter()
                .any(|argument| argument.as_expression().is_none())
        {
            self.record_unknown_lock_helper(call, name.name.as_str());
            return;
        }
        for (executor, _, query) in helper.calls {
            let Some(receiver) = call
                .arguments
                .get(executor)
                .and_then(|arg| arg.as_expression())
            else {
                continue;
            };
            let receiver = match unwrap_ts_wrappers(receiver) {
                Expression::StaticMemberExpression(member) if member.property.name == "query" => {
                    &member.object
                }
                receiver => receiver,
            };
            let mut fact = EmbeddedSqlCall::default();
            super::transaction::annotate_receiver(self, receiver, call.span.start, &mut fact);
            if !fact.executor_transaction_known || fact.transaction_id.is_none() {
                continue;
            }
            let Some(sql) = query
                .arguments
                .first()
                .and_then(|arg| arg.as_expression())
                .and_then(crate::codebase::postgres::embedded::sql_text)
            else {
                continue;
            };
            fact.sql_text = Some(sql);
            fact.kind = EmbeddedSqlKind::Inline;
            fact.lock_scope_analysis = true;
            fact.line = byte_offset_to_line(self.source, call.span.start as usize);
            fact.callee = name.name.to_string();
            fact.execution_unconditional = self.helper_execution_unconditional();
            fact.lock_helper = Some(Box::new((
                name.name.to_string(),
                byte_offset_to_line(self.source, query.span.start as usize),
            )));
            if let Some(Expression::ArrayExpression(array)) = query
                .arguments
                .get(1)
                .and_then(|arg| arg.as_expression())
                .map(unwrap_ts_wrappers)
            {
                fact.bound_parameters = array
                    .elements
                    .iter()
                    .map(|item| {
                        let expression = item.as_expression()?;
                        if let Expression::Identifier(id) = unwrap_ts_wrappers(expression) {
                            let index = helper
                                .params
                                .iter()
                                .position(|param| *param == id.name.as_str())?;
                            return call
                                .arguments
                                .get(index)
                                .and_then(|arg| arg.as_expression())
                                .and_then(|value| super::transaction::identity(self, value));
                        }
                        super::transaction::literal_identity(expression)
                    })
                    .collect();
            }
            self.applied_lock_helper_spans
                .insert((query.span.start, query.span.end));
            self.calls.push(fact);
            self.call_spans.push((call.span.start, call.span.end));
            if self.track_order {
                self.confirmed_order.push(self.next_seq);
            }
            self.next_seq += 1;
        }
    }
    pub(super) fn mark_lock_helper_definitions(&mut self) {
        for (call, span) in self.calls.iter_mut().zip(&self.call_spans) {
            call.helper_definition = self.applied_lock_helper_spans.contains(span);
        }
    }
}

impl ScopeVisitor<'_> {
    pub(super) fn lock_helper_summaries(&mut self) -> FxHashMap<String, Summary> {
        #[cfg(test)]
        {
            self.lock_summary_passes += 1;
        }
        summary::summaries(&self.lock_helpers, self.source)
    }
}
