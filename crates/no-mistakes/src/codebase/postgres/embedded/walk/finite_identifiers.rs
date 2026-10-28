//! Finite runtime keys from an exact imported Vitest `it.each` callback.
mod case;
mod import_uses;
mod map;
mod uses;
use super::{resolve::functions::reassigned::ReassignedNames, ScopeVisitor};
use crate::fx::FxHashSet;
pub(in crate::codebase::postgres::embedded::walk) use case::FiniteCase;
use oxc_ast::ast::{
    CallExpression, ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement,
};
use oxc_ast_visit::Visit;

pub(in crate::codebase::postgres::embedded::walk) fn vitest_it_imports(
    program: &Program<'_>,
) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind == ImportOrExportKind::Type
            || crate::ast::js_literal(&import.source).as_ref() != "vitest"
        {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                continue;
            };
            if named.import_kind != ImportOrExportKind::Type
                && crate::ast::module_export_name(&named.imported) == "it"
            {
                names.insert(named.local.name.to_string());
            }
        }
    }
    if names.is_empty() {
        return names;
    }
    let reassigned = ReassignedNames::collect(program);
    names.retain(|name| !reassigned.contains(name));
    if names.is_empty() {
        return names;
    }
    let mut uses = import_uses::ImportUses::new(&names);
    uses.visit_program(program);
    names.retain(|name| !uses.invalid.contains(name));
    names
}

impl<'a> ScopeVisitor<'a> {
    pub(super) fn visit_finite_call(&mut self, call: &CallExpression<'a>) {
        self.poison_literal_callee(call);
        let finite = (!self.head_calls_only)
            .then(|| {
                FiniteCase::from_call(
                    call,
                    &self.vitest_it_imports,
                    |name| self.shadowed_locally(name),
                    self.function_scopes.len(),
                )
            })
            .flatten();
        let inserted = finite.is_some();
        if let Some(finite) = finite {
            self.finite_cases.push(finite);
        }
        self.visit_sql_call(call);
        if inserted {
            self.finite_cases.pop();
        }
    }
}
