mod calls;
mod literals;

use super::{resolve, ScopeVisitor};
use calls::walk_calls;
use literals::{is_direct_function, is_plain_literal};
use oxc_ast::ast::{Expression, ReturnStatement};
use oxc_span::GetSpan;

type FunctionKey = (u32, String);

#[derive(Default)]
pub(super) struct PlainState {
    pub(super) pending_name: Option<FunctionKey>,
    names: Vec<Option<FunctionKey>>,
    reached: crate::fx::FxHashSet<FunctionKey>,
    returns: Vec<DeferredReturn>,
    edges: Vec<ReturnEdge>,
    aliases: crate::fx::FxHashMap<FunctionKey, FunctionKey>,
}

struct DeferredReturn {
    owner: FunctionKey,
    line: u32,
    sql: String,
}

struct ReturnEdge {
    owner: FunctionKey,
    calls: Vec<FunctionKey>,
}

impl ScopeVisitor<'_> {
    pub(super) fn enter_plain_function(&mut self, explicit: Option<&str>) {
        let pending = self.plain.pending_name.take();
        let name = pending.or_else(|| explicit.map(|name| self.plain_key(name)));
        self.plain.names.push(name);
    }

    pub(super) fn leave_plain_function(&mut self) {
        self.plain.names.pop();
    }

    pub(super) fn arm_plain_binding(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclarator<'_>,
    ) -> Option<FunctionKey> {
        let saved = self.plain.pending_name.take();
        if let (oxc_ast::ast::BindingPattern::BindingIdentifier(name), Some(init)) =
            (&declaration.id, &declaration.init)
        {
            if is_direct_function(init) {
                self.plain.pending_name = Some(self.plain_key(name.name.as_str()));
            }
        }
        saved
    }

    pub(super) fn restore_plain_binding(&mut self, saved: Option<FunctionKey>) {
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
        let mut keys = Vec::new();
        walk_calls(expression, &mut |name| keys.push(self.plain_key(name)));
        self.plain.reached.extend(keys);
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
    fn current_plain_name(&self) -> Option<&FunctionKey> {
        self.plain.names.last().and_then(Option::as_ref)
    }

    pub(super) fn alias_plain_self(&mut self, name: &str) {
        if let Some(owner) = self.current_plain_name().cloned() {
            let key = (
                *self.member_query_proof.scope_ids.last().expect("scope"),
                name.to_string(),
            );
            self.plain.aliases.insert(key, owner);
        }
    }

    fn plain_key(&self, name: &str) -> FunctionKey {
        let index = self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
            .unwrap_or(0);
        let key = (self.member_query_proof.scope_ids[index], name.to_string());
        self.plain.aliases.get(&key).cloned().unwrap_or(key)
    }
    fn defer_plain_return(&mut self, start: u32, sql: String) {
        let line = crate::codebase::ts_source::byte_offset_to_line(self.source, start as usize);
        if let Some(owner) = self.current_plain_name() {
            self.plain.returns.push(DeferredReturn {
                owner: owner.clone(),
                line,
                sql,
            });
        } else {
            // Member calls have no lexical callable key: retain recall conservatively.
            self.push_fragment(line, Some(sql), None, None);
        }
    }

    fn note_returned_calls(&mut self, argument: &Expression<'_>) {
        let Some(owner) = self.current_plain_name() else {
            return;
        };
        let mut calls = Vec::new();
        walk_calls(argument, &mut |name| calls.push(self.plain_key(name)));
        if calls.is_empty() {
            return;
        }
        self.plain.edges.push(ReturnEdge {
            owner: owner.clone(),
            calls,
        });
    }
}
