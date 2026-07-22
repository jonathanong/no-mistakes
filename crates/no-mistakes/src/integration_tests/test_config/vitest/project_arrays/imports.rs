use oxc_ast::ast::{ImportDeclarationSpecifier, Program, Statement};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(super) struct ImportBinding {
    pub(super) source: String,
    pub(super) imported: String,
}

pub(super) fn import_bindings(program: &Program<'_>) -> BTreeMap<String, ImportBinding> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let (local, imported) = match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    (specifier.local.name.to_string(), "default".to_string())
                }
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if specifier.import_kind.is_type() =>
                {
                    continue;
                }
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                    specifier.local.name.to_string(),
                    specifier.imported.name().to_string(),
                ),
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    (specifier.local.name.to_string(), "*".to_string())
                }
            };
            bindings.insert(
                local,
                ImportBinding {
                    source: import.source.value.to_string(),
                    imported,
                },
            );
        }
    }
    bindings
}

/// Runtime module sources, including side-effect imports and re-exports.
/// Dynamic Vitest setup values use this for a bounded helper-module closure.
pub(super) fn import_sources(program: &Program<'_>) -> BTreeSet<String> {
    program
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::ImportDeclaration(import)
                if crate::fetch::import_shape::is_runtime_import(import) =>
            {
                Some(import.source.value.to_string())
            }
            Statement::ExportNamedDeclaration(export)
                if crate::fetch::import_shape::is_runtime_export(export) =>
            {
                export
                    .source
                    .as_ref()
                    .map(|source| source.value.to_string())
            }
            Statement::ExportAllDeclaration(export) if !export.export_kind.is_type() => {
                Some(export.source.value.to_string())
            }
            _ => None,
        })
        .collect()
}
