use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{BindingPattern, Expression, Function, Program, VariableDeclarator};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) fn collect(program: &Program<'_>) -> FxHashSet<String> {
    let mut names = Names::default();
    names.visit_program(program);
    names.0
}

#[derive(Default)]
struct Names(FxHashSet<String>);

impl<'a> Visit<'a> for Names {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if function.body.is_some() {
            if let Some(id) = &function.id {
                self.0.insert(id.name.to_string());
            }
        }
        walk::walk_function(self, function, flags);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let (BindingPattern::BindingIdentifier(ident), Some(init)) =
            (&declarator.id, &declarator.init)
        {
            if function_value(init) {
                self.0.insert(ident.name.to_string());
            }
        }
        walk::walk_variable_declarator(self, declarator);
    }
}

fn function_value(init: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(init) {
        Expression::FunctionExpression(function) => function.body.is_some(),
        Expression::ArrowFunctionExpression(_) => true,
        _ => false,
    }
}

impl super::LocalFunctions {
    pub(in crate::codebase::postgres::embedded::walk) fn declares_function_body(
        &self,
        name: &str,
    ) -> bool {
        self.function_bodies.contains(name)
    }
}
