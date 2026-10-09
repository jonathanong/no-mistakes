use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;

pub(super) fn collect(statements: &[Statement<'_>]) -> HashSet<String> {
    let mut names = Names::default();
    for statement in statements {
        names.visit_statement(statement);
    }
    names.0
}

#[derive(Default)]
struct Names(HashSet<String>);
impl<'a> Visit<'a> for Names {
    fn visit_variable_declaration(&mut self, value: &VariableDeclaration<'a>) {
        if value.kind == VariableDeclarationKind::Var {
            for value in &value.declarations {
                super::bindings::bound(&value.id, &mut self.0);
            }
        }
        walk::walk_variable_declaration(self, value);
    }
    // Hoisted bindings never escape these runtime scope boundaries.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
    fn visit_class(&mut self, _: &Class<'a>) {}
    fn visit_ts_module_block(&mut self, _: &TSModuleBlock<'a>) {}
}
