use super::super::options::EmbeddedSqlOptions;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement};
use std::collections::HashMap;

pub(super) fn configured_namespaces(
    program: &Program<'_>,
    options: &EmbeddedSqlOptions,
) -> HashMap<String, String> {
    if options.import_specifier.is_empty() {
        return HashMap::new();
    }
    let mut names = HashMap::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind == ImportOrExportKind::Type
            || !super::super::scoped_bindings::from_configured_module(
                crate::ast::js_literal(&import.source).as_ref(),
                &options.import_specifier,
            )
        {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            if let ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) = specifier {
                names.insert(
                    namespace.local.name.to_string(),
                    crate::ast::js_literal(&import.source).into_owned(),
                );
            }
        }
    }
    names
}

pub(super) fn configured_values(
    program: &Program<'_>,
    options: &EmbeddedSqlOptions,
    bindings: &std::collections::HashSet<String>,
) -> HashMap<String, (String, String)> {
    let mut names = HashMap::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind == ImportOrExportKind::Type
            || (!options.import_specifier.is_empty()
                && crate::ast::js_literal(&import.source).as_ref() != options.import_specifier)
        {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                continue;
            };
            if named.import_kind == ImportOrExportKind::Type
                || !bindings.contains(named.local.name.as_str())
            {
                continue;
            }
            names.insert(
                named.local.name.to_string(),
                (
                    crate::ast::js_literal(&import.source).into_owned(),
                    crate::ast::module_export_name(&named.imported).to_string(),
                ),
            );
        }
    }
    names
}
