//! Import trust and writes to module bindings, extracted from the shared AST.
mod assigned;
mod bindings;
mod conventional;
mod hoisted;
mod local;
mod visitor;
use oxc_ast::ast::{ImportDeclarationSpecifier, Program, Statement};
use std::collections::{BTreeSet, HashSet};

pub(super) fn collect(
    program: &Program<'_>,
    options: &super::super::EmbeddedSqlOptions,
) -> BTreeSet<String> {
    let mut trusted = conventional::collect(program);
    // Runtime module bindings shadow the conventional global tag, including
    // destructuring and TypeScript import-equals declarations.
    for name in bindings::function_scope(&program.body) {
        trusted.remove(&name);
    }
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind == oxc_ast::ast::ImportOrExportKind::Type {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            if matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(value)
                if value.import_kind == oxc_ast::ast::ImportOrExportKind::Type)
            {
                continue;
            }
            let local = match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(value) => &value.local,
                ImportDeclarationSpecifier::ImportSpecifier(value) => &value.local,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(value) => &value.local,
            };
            trusted.remove(local.name.as_str());
            let valid = match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                    import.source.value == "sql-template-strings"
                }
                ImportDeclarationSpecifier::ImportSpecifier(value) => {
                    (import.source.value == "sql-template-strings"
                        && (value.imported.name() == "default"
                            || (value.imported.name() == "sql"
                                && value.local.name.eq_ignore_ascii_case("sql"))))
                        || super::super::embedded::matches_trusted_sql_import(
                            import.source.value.as_str(),
                            value.imported.name().as_str(),
                            &options.trusted_sql_tags,
                        )
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => false,
            };
            if valid {
                trusted.insert(local.name.to_string());
            }
        }
    }
    trusted
}

pub(super) fn legacy_local_tags(program: &Program<'_>) -> std::collections::BTreeMap<String, u32> {
    local::collect(program)
}

/// Nested lexical declarations shadow imports; only writes resolving to the
/// module binding invalidate that binding's helper body or trusted SQL tag.
pub(super) fn reassigned(program: &Program<'_>) -> HashSet<String> {
    visitor::collect(program)
}

#[cfg(test)]
mod tests;

/// Runtime declarations reserve lexical names without replacing trusted imports.
pub(super) fn declared_names(statements: &[Statement<'_>]) -> HashSet<String> {
    bindings::declarations(statements)
}

pub(super) fn bound_names(pattern: &oxc_ast::ast::BindingPattern<'_>) -> Vec<String> {
    let mut names = HashSet::new();
    bindings::bound(pattern, &mut names);
    let mut names: Vec<_> = names.into_iter().collect();
    names.sort();
    names
}
