use super::{resolve, scope, BindingState, ScopeVisitor};
use oxc_ast::ast::{ReturnStatement, TryStatement};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use std::collections::HashMap;

impl ScopeVisitor<'_> {
    pub(super) fn invalidate_execution_variants(&mut self) {
        let owner = self.function_scopes.last().copied().unwrap_or(0);
        for scope in &mut self.scopes[owner..] {
            for binding in scope.values_mut() {
                binding.variants = None;
            }
        }
    }
}

pub(super) fn try_statement<'a>(visitor: &mut ScopeVisitor<'a>, statement: &TryStatement<'a>) {
    let mut before = visitor.scopes.clone();
    visitor.visit_block_statement(&statement.block);
    invalidate_exception_state(visitor, &before);
    if let Some(handler) = &statement.handler {
        before = visitor.scopes.clone();
        visitor.visit_catch_clause(handler);
        invalidate_exception_state(visitor, &before);
    }
    if let Some(finalizer) = &statement.finalizer {
        before = visitor.scopes.clone();
        visitor.visit_block_statement(finalizer);
        invalidate_exception_state(visitor, &before);
    }
}

fn invalidate_exception_state(
    visitor: &mut ScopeVisitor<'_>,
    before: &[HashMap<String, BindingState>],
) {
    let owner = visitor.function_scopes.last().copied().unwrap_or(0);
    for (index, bindings) in visitor.scopes.iter_mut().enumerate().skip(owner) {
        for (name, binding) in bindings {
            let changed = before
                .get(index)
                .and_then(|scope| scope.get(name))
                .is_none_or(|old| {
                    (
                        &old.sql,
                        old.kind,
                        old.builder_identity,
                        &old.variants,
                        old.builder_effect_revision,
                    ) != (
                        &binding.sql,
                        binding.kind,
                        binding.builder_identity,
                        &binding.variants,
                        binding.builder_effect_revision,
                    )
                });
            // A handler can see any earlier point in the preceding phase,
            // including the value before its last successful mutation.
            if changed {
                scope::mark_binding_dynamic_keep_known_statement(binding);
            }
            binding.variants = None;
        }
    }
}
pub(super) fn returned<'a>(visitor: &mut ScopeVisitor<'a>, statement: &ReturnStatement<'a>) {
    let fragment = statement.argument.as_ref().and_then(|argument| {
        resolve::builder_fragment(argument, visitor).map(|sql| (argument.span().start, sql))
    });
    if let Some((start, sql)) = fragment {
        visitor.push_fragment(
            crate::codebase::ts_source::byte_offset_to_line(visitor.source, start as usize),
            Some(sql),
            statement.argument.as_ref(),
            None,
        );
        visitor.suppress_nested_builder_fragments += 1;
        walk::walk_return_statement(visitor, statement);
        visitor.suppress_nested_builder_fragments -= 1;
    } else {
        walk::walk_return_statement(visitor, statement);
    }
    visitor.invalidate_execution_variants();
}
