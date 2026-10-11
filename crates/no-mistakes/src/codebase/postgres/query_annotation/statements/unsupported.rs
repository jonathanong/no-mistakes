//! Preserve potential calls in unsupported control flow without interpreting it.
use oxc_ast::ast::{CallExpression, Function, Statement};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) fn calls(statement: &Statement<'_>) -> Vec<(u32, u32)> {
    #[derive(Default)]
    struct Calls(Vec<(u32, u32)>);
    impl<'a> Visit<'a> for Calls {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            self.0.push((call.span.start, call.span.end));
            walk::walk_call_expression(self, call);
        }
        fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
        fn visit_arrow_function_expression(
            &mut self,
            _: &oxc_ast::ast::ArrowFunctionExpression<'a>,
        ) {
        }
    }
    let mut calls = Calls::default();
    calls.visit_statement(statement);
    calls.0
}
