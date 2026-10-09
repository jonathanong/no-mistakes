//! Ordinary sloppy functions with simple parameters have mapped arguments.
//! OXC's lexical scope flags carry module, directive, arrow, and class strictness.
use crate::fx::FxHashSet;
use oxc_ast::ast::{BindingPattern, Function, Program};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use std::cell::Cell;

#[derive(Default)]
struct MappedArguments {
    strict: Vec<bool>,
    functions: FxHashSet<u32>,
}

impl MappedArguments {
    fn inherited_strict(&self) -> bool {
        self.strict.last().copied().unwrap_or_default()
    }
}

impl<'a> Visit<'a> for MappedArguments {
    fn enter_scope(&mut self, flags: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.strict
            .push(self.inherited_strict() || flags.is_strict_mode());
    }

    fn leave_scope(&mut self) {
        self.strict.pop();
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if !self.inherited_strict()
            && !function.has_use_strict_directive()
            && function.params.rest.is_none()
            && function.params.items.iter().all(|param| {
                matches!(param.pattern, BindingPattern::BindingIdentifier(_))
                    && param.initializer.is_none()
            })
        {
            self.functions.insert(function.span.start);
        }
        walk::walk_function(self, function, flags);
    }
}

pub(super) fn collect(program: &Program<'_>) -> FxHashSet<u32> {
    let mut visitor = MappedArguments::default();
    visitor.visit_program(program);
    visitor.functions
}

#[cfg(test)]
mod tests;
