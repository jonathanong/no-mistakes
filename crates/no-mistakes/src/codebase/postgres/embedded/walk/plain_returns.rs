mod calls;

use super::{resolve, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use calls::walk_calls;
use oxc_ast::ast::{Expression, ReturnStatement};
use oxc_span::GetSpan;

#[derive(Default)]
pub(super) struct PlainState {
    pub(super) pending_name: Option<String>,
    names: Vec<Option<String>>,
    reached: crate::fx::FxHashSet<String>,
    returns: Vec<DeferredReturn>,
    edges: Vec<ReturnEdge>,
}

struct DeferredReturn {
    owner: String,
    line: u32,
    sql: String,
}

struct ReturnEdge {
    owner: String,
    calls: Vec<String>,
}

impl ScopeVisitor<'_> {
    pub(super) fn enter_plain_function(&mut self, explicit: Option<&str>) {
        let pending = self.plain.pending_name.take();
        let name = explicit.map(str::to_string).or(pending);
        self.plain.names.push(name);
    }

    pub(super) fn leave_plain_function(&mut self) {
        self.plain.names.pop();
    }

    pub(super) fn arm_plain_binding(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclarator<'_>,
    ) -> Option<String> {
        let saved = self.plain.pending_name.take();
        if let (oxc_ast::ast::BindingPattern::BindingIdentifier(name), Some(init)) =
            (&declaration.id, &declaration.init)
        {
            if is_direct_function(init) {
                self.plain.pending_name = Some(name.name.to_string());
            }
        }
        saved
    }

    pub(super) fn restore_plain_binding(&mut self, saved: Option<String>) {
        self.plain.pending_name = saved;
    }

    /// `true` when the caller should hide nested fragments while walking the return.
    pub(super) fn consume_return(&mut self, statement: &ReturnStatement<'_>) -> bool {
        let Some(argument) = &statement.argument else {
            return false;
        };
        let Some(sql) = resolve::builder_fragment(argument, self) else {
            self.note_returned_calls(argument);
            return false;
        };
        if is_plain_literal(argument) {
            self.defer_plain_return(argument.span().start, sql);
            return false;
        }
        let line = crate::codebase::ts_source::byte_offset_to_line(
            self.source,
            argument.span().start as usize,
        );
        self.push_fragment(line, Some(sql), Some(argument), None);
        true
    }

    pub(super) fn note_executor_call(&mut self, call: &oxc_ast::ast::CallExpression<'_>) {
        for argument in &call.arguments {
            if let Some(expression) = argument.as_expression() {
                self.note_sql_expression(expression);
            }
        }
    }

    pub(super) fn note_builder_append(&mut self, call: &oxc_ast::ast::CallExpression<'_>) {
        if !resolve::is_builder_append(call, self) {
            return;
        }
        if let Some(argument) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
        {
            self.note_sql_expression(argument);
        }
    }

    pub(super) fn note_sql_expression(&mut self, expression: &Expression<'_>) {
        let reached = &mut self.plain.reached;
        walk_calls(expression, &mut |name| {
            reached.insert(name.to_string());
        });
    }

    pub(super) fn flush_plain_returns(&mut self) {
        let mut reached = std::mem::take(&mut self.plain.reached);
        loop {
            let mut grew = false;
            for edge in &self.plain.edges {
                if !reached.contains(&edge.owner) {
                    continue;
                }
                for callee in &edge.calls {
                    if reached.insert(callee.clone()) {
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
        let pending = std::mem::take(&mut self.plain.returns);
        for item in pending {
            if reached.contains(&item.owner) {
                self.push_fragment(item.line, Some(item.sql), None, None);
            }
        }
    }

    fn current_plain_name(&self) -> Option<&str> {
        self.plain.names.last().and_then(|name| name.as_deref())
    }

    fn defer_plain_return(&mut self, start: u32, sql: String) {
        let Some(owner) = self.current_plain_name() else {
            return;
        };
        let line = crate::codebase::ts_source::byte_offset_to_line(self.source, start as usize);
        self.plain.returns.push(DeferredReturn {
            owner: owner.to_string(),
            line,
            sql,
        });
    }

    fn note_returned_calls(&mut self, argument: &Expression<'_>) {
        let Some(owner) = self.current_plain_name() else {
            return;
        };
        let mut calls = Vec::new();
        walk_calls(argument, &mut |name| calls.push(name.to_string()));
        if calls.is_empty() {
            return;
        }
        self.plain.edges.push(ReturnEdge {
            owner: owner.to_string(),
            calls,
        });
    }
}

fn is_direct_function(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expression),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}

fn is_plain_literal(expression: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}
