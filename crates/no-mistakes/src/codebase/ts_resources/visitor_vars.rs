use super::bindings::binding_names;
use super::ResourceVisitor;
use oxc_ast::ast::{Function, Statement, VariableDeclaration, VariableDeclarationKind};
use oxc_ast_visit::{walk, Visit};

/// Collect `var` declarations beneath control-flow statements without entering
/// nested functions, whose declarations belong to their own function scopes.
#[derive(Default)]
struct VarBindingCollector {
    names: Vec<String>,
}

impl<'a> Visit<'a> for VarBindingCollector {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                self.names.extend(binding_names(&declarator.id));
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }

    fn visit_function(&mut self, _: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}
}

fn var_binding_names(statements: &[Statement<'_>]) -> Vec<String> {
    let mut collector = VarBindingCollector::default();
    for statement in statements {
        collector.visit_statement(statement);
    }
    collector.names.sort();
    collector.names.dedup();
    collector.names
}

impl<'a> ResourceVisitor<'a> {
    /// `var` is function-scoped even when it appears in an `if`, `try`, or
    /// loop body. Predeclare it before the first call in that function (or
    /// program) so hoisted local bindings cannot be mistaken for imports.
    pub(super) fn predeclare_var_bindings_in_statements(&mut self, statements: &[Statement<'_>]) {
        for name in var_binding_names(statements) {
            self.declare_var_binding(&name, None);
        }
    }

    pub(super) fn predeclare_function_var_bindings(&mut self, function: &Function<'_>) {
        let Some(body) = &function.body else { return };
        self.predeclare_var_bindings_in_statements(&body.statements);
    }
}
