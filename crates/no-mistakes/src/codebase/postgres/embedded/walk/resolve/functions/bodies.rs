use super::super::for_each_bound_name;
use crate::fx::FxHashSet;
use oxc_ast::ast::{
    FormalParameters, Function, ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind,
    Program, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

/// Names whose callee is outside this file: a top-level import or an ambient
/// `declare function`, minus any other binding of that name.
pub(super) fn collect(program: &Program<'_>) -> FxHashSet<String> {
    let mut names = Names::default();
    names.visit_program(program);
    let Names {
        mut external,
        local,
    } = names;
    // A same-file function, parameter, const, or let hides the external name.
    external.retain(|name| !local.contains(name));
    external
}

#[derive(Default)]
struct Names {
    external: FxHashSet<String>,
    local: FxHashSet<String>,
}

impl Names {
    fn bind_pattern(&mut self, pattern: &oxc_ast::ast::BindingPattern<'_>) {
        for_each_bound_name(pattern, &mut |name| {
            self.local.insert(name.to_string());
        });
    }
}

impl<'a> Visit<'a> for Names {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        if import.import_kind == ImportOrExportKind::Type {
            return;
        }
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            if let Some(name) = import_value_local(specifier) {
                self.external.insert(name.to_string());
            }
        }
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if let Some(id) = &function.id {
            // Overload signatures also have no body. Only `declare` is ambient.
            let slot = if function.declare {
                &mut self.external
            } else {
                &mut self.local
            };
            slot.insert(id.name.to_string());
        }
        walk::walk_function(self, function, flags);
    }

    fn visit_formal_parameters(&mut self, params: &FormalParameters<'a>) {
        for param in &params.items {
            self.bind_pattern(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.bind_pattern(&rest.rest.argument);
        }
        walk::walk_formal_parameters(self, params);
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        for declarator in &declaration.declarations {
            self.bind_pattern(&declarator.id);
        }
        walk::walk_variable_declaration(self, declaration);
    }
}

fn import_value_local<'a>(specifier: &'a ImportDeclarationSpecifier<'a>) -> Option<&'a str> {
    match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(named) => {
            (named.import_kind != ImportOrExportKind::Type).then_some(named.local.name.as_str())
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
            Some(default.local.name.as_str())
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
            Some(namespace.local.name.as_str())
        }
    }
}

impl super::LocalFunctions {
    pub(in crate::codebase::postgres::embedded::walk) fn declared_elsewhere(
        &self,
        name: &str,
    ) -> bool {
        self.declared_elsewhere.contains(name)
    }
}
