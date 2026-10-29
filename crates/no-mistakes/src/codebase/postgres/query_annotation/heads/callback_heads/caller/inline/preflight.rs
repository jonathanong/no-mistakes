//! Negative-only endpoint preflight before constructing callback proof indexes.
use super::{endpoint, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, BindingPattern, Expression};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(in super::super) fn may_execute(value: &Expression<'_>, executors: &FxHashSet<Span>) -> bool {
    let Expression::ArrowFunctionExpression(arrow) = unwrap_ts_wrappers(value) else {
        return false;
    };
    if arrow.params.rest.is_some() || arrow.params.items.len() != 1 {
        return false;
    }
    let param = &arrow.params.items[0];
    if param.initializer.is_some() || !param.decorators.is_empty() {
        return false;
    }
    let BindingPattern::BindingIdentifier(param) = &param.pattern else {
        return false;
    };
    if matches!(param.name.as_str(), "arguments" | "eval") {
        return false;
    }
    let mut finder = Finder {
        name: param.name.as_str(),
        executors,
        found: false,
    };
    finder.visit_arrow_function_body(&arrow.body);
    finder.found
}

struct Finder<'a> {
    name: &'a str,
    executors: &'a FxHashSet<Span>,
    found: bool,
}
impl<'a> Visit<'a> for Finder<'_> {
    fn visit_call_expression(&mut self, call: &ast::CallExpression<'a>) {
        self.found |= endpoint::matching(call, self.name, self.executors).is_some();
        if !self.found {
            walk::walk_call_expression(self, call);
        }
    }
    // The full census rejects nested functions/classes. Their endpoints cannot
    // turn the outer callback into a candidate or demand file-wide indexes.
    fn visit_function(&mut self, _: &ast::Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ast::ArrowFunctionExpression<'a>) {}
    fn visit_class(&mut self, _: &ast::Class<'a>) {}
}
