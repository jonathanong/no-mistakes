use super::{
    annotation_status_for_call, callback_argument_index, describe_name, hook_callback,
    merge_annotation_status, status_for_if_branch, test_callback_traversal, HookKind,
    TestOccurrenceScope, TestStatus,
};
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{
    CallExpression, ConditionalExpression, IfStatement, LogicalExpression, NewExpression, Program,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashMap;

/// Per-call line context derived from a single AST walk: the merged
/// `TestStatus` (Active / Conditional / Skipped) and the structural
/// `TestOccurrenceScope` (File / Hook / TeardownHook / Test) at the point of
/// the call. Consumers like the diff-aware HTTP and queue hint scanners
/// extract their own call data via simpler walkers and then look up each
/// call's line here to apply the same `TestPolicy` and `TeardownHook` filter
/// the selector / route reverse indexes already use.
#[derive(Clone, Copy, Debug)]
pub struct CallContext {
    pub status: TestStatus,
    pub scope: TestOccurrenceScope,
}

/// Build a `line → CallContext` map by visiting every call expression in
/// `program` and recording the test-context that was active at that call.
/// Lines that map to multiple contexts (nested calls on the same source
/// line) keep the most restrictive context — Skipped > Conditional > Active,
/// and TeardownHook > Hook > Test > File — so the diff-aware filter never
/// over-counts.
pub fn build_call_context_index(program: &Program<'_>, source: &str) -> HashMap<u32, CallContext> {
    let mut visitor = CallContextVisitor {
        source,
        status: TestStatus::Active,
        annotation_status: TestStatus::Active,
        current_scope: TestOccurrenceScope::File,
        contexts: HashMap::new(),
    };
    visitor.visit_program(program);
    visitor.contexts
}

struct CallContextVisitor<'a> {
    source: &'a str,
    status: TestStatus,
    annotation_status: TestStatus,
    current_scope: TestOccurrenceScope,
    contexts: HashMap<u32, CallContext>,
}

impl<'a> Visit<'a> for CallContextVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let line = byte_offset_to_line(self.source, call.span.start as usize);
        self.record(line);

        let traversal = test_callback_traversal(call, self.annotation_status);
        if let Some((callback_index, callback_status)) = traversal {
            if describe_name(call).is_some() {
                for (index, argument) in call.arguments.iter().enumerate() {
                    if index == callback_index {
                        self.with_status(callback_status, |visitor| {
                            visitor.with_annotation_scope(|visitor| {
                                visitor.visit_argument(argument);
                            });
                        });
                    } else {
                        self.visit_argument(argument);
                    }
                }
            } else {
                let previous_scope = self.current_scope;
                self.current_scope = TestOccurrenceScope::Test;
                for (index, argument) in call.arguments.iter().enumerate() {
                    if index == callback_index {
                        self.with_status(callback_status, |visitor| {
                            visitor.with_annotation_scope(|visitor| {
                                visitor.visit_argument(argument);
                            });
                        });
                    } else {
                        self.visit_argument(argument);
                    }
                }
                self.current_scope = previous_scope;
            }
            return;
        }

        let callback_index = callback_argument_index(call);
        if annotation_status_for_call(call).is_some() {
            self.apply_annotation_call(call);
            for (index, argument) in call.arguments.iter().enumerate() {
                if Some(index) != callback_index {
                    self.visit_argument(argument);
                }
            }
            return;
        }
        if let Some((callback_index, hook_kind)) = hook_callback(call) {
            let previous_scope = self.current_scope;
            self.current_scope = match hook_kind {
                HookKind::Setup => TestOccurrenceScope::Hook,
                HookKind::Teardown => TestOccurrenceScope::TeardownHook,
            };
            self.visit_argument(&call.arguments[callback_index]);
            self.current_scope = previous_scope;
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_if_statement(&mut self, statement: &IfStatement<'a>) {
        self.visit_expression(&statement.test);
        let status = status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_statement(&statement.consequent);
            if let Some(alternate) = &statement.alternate {
                visitor.visit_statement(alternate);
            }
        });
    }

    fn visit_conditional_expression(&mut self, expression: &ConditionalExpression<'a>) {
        self.visit_expression(&expression.test);
        let status = status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.consequent);
            visitor.visit_expression(&expression.alternate);
        });
    }

    fn visit_logical_expression(&mut self, expression: &LogicalExpression<'a>) {
        self.visit_expression(&expression.left);
        let status = status_for_if_branch(self.status);
        self.with_status(status, |visitor| {
            visitor.visit_expression(&expression.right)
        });
    }

    fn visit_new_expression(&mut self, expression: &NewExpression<'a>) {
        // Constructor calls like `new Worker('queue', ...)` and
        // `new Queue('emails')` aren't `CallExpression`s, but the dependent-
        // side queue scan keys hints by the line of those `new` calls. Record
        // their line so a worker declared inside `test.skip(...)` or an
        // `afterEach` block doesn't slip past the `TestPolicy` /
        // `TeardownHook` filter.
        let line = byte_offset_to_line(self.source, expression.span.start as usize);
        self.record(line);
        walk::walk_new_expression(self, expression);
    }
}

impl CallContextVisitor<'_> {
    fn record(&mut self, line: u32) {
        let merged_status = self.status.merge(self.annotation_status);
        let candidate = CallContext {
            status: merged_status,
            scope: self.current_scope,
        };
        self.contexts
            .entry(line)
            .and_modify(|existing| {
                if scope_rank(candidate.scope) > scope_rank(existing.scope) {
                    existing.scope = candidate.scope;
                }
                existing.status = existing.status.merge(candidate.status);
            })
            .or_insert(candidate);
    }

    fn with_status(&mut self, status: TestStatus, visit: impl FnOnce(&mut Self)) {
        let previous = self.status;
        self.status = previous.merge(status);
        visit(self);
        self.status = previous;
    }

    fn with_annotation_scope(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = self.annotation_status;
        self.annotation_status = TestStatus::Active;
        visit(self);
        self.annotation_status = previous;
    }

    fn apply_annotation_call(&mut self, call: &CallExpression<'_>) {
        if let Some(status) = annotation_status_for_call(call) {
            let status = merge_annotation_status(self.status, status);
            self.annotation_status = merge_annotation_status(self.annotation_status, status);
        }
    }
}

fn scope_rank(scope: TestOccurrenceScope) -> u8 {
    match scope {
        TestOccurrenceScope::File => 0,
        TestOccurrenceScope::Test => 1,
        TestOccurrenceScope::Hook => 2,
        TestOccurrenceScope::TeardownHook => 3,
    }
}

#[cfg(test)]
#[path = "call_context_index/tests.rs"]
mod tests;
