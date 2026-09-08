use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, Program, Statement, VariableDeclaration,
    VariableDeclarator,
};
use std::collections::HashSet;

/// Top-level `const`/`let`/`var` bindings whose initializer isn't itself a
/// same-file callable helper shape (`function`/arrow expression) — anything
/// else rebinds the name away from whatever a same-file helper's body would
/// otherwise trust it to mean when referencing it through closure (not
/// through the helper's own parameters, which `shadows_param` already
/// covers). A top-level `function` declaration is out of scope here: that
/// shape is a legitimate same-file helper collected elsewhere, including one
/// referencing its own top-level declaration by name.
#[derive(Default)]
pub(super) struct TagShadows {
    names: HashSet<String>,
}

impl TagShadows {
    pub(super) fn collect(program: &Program<'_>) -> Self {
        let mut shadows = Self::default();
        for statement in &program.body {
            record_statement(statement, &mut shadows);
        }
        shadows
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

fn record_statement(statement: &Statement<'_>, shadows: &mut TagShadows) {
    match statement {
        Statement::VariableDeclaration(declaration) => record_declaration(declaration, shadows),
        Statement::ExportDeclaration(export) => {
            if let Declaration::VariableDeclaration(declaration) = &export.declaration {
                record_declaration(declaration, shadows);
            }
        }
        _ => {}
    }
}

fn record_declaration(declaration: &VariableDeclaration<'_>, shadows: &mut TagShadows) {
    for declarator in &declaration.declarations {
        record_declarator(declarator, shadows);
    }
}

fn record_declarator(declarator: &VariableDeclarator<'_>, shadows: &mut TagShadows) {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return;
    };
    let is_helper_shape = declarator
        .init
        .as_ref()
        .is_some_and(|init| is_function_shaped(init));
    if !is_helper_shape {
        shadows.names.insert(ident.name.to_string());
    }
}

fn is_function_shaped(expr: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expr),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}
