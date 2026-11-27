use super::{binding_name, Binding};
use crate::codebase::ts_resolver::{normalize_path, resolve_import, TsConfig};
use oxc_ast::ast::{
    Declaration, ImportDeclarationSpecifier, ImportOrExportKind, ModuleExportName, Program,
    Statement, VariableDeclarationKind,
};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn collect_imports(
    program: &Program<'_>,
    importer: &Path,
    bindings: &mut BTreeMap<String, Binding>,
) {
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind != ImportOrExportKind::Value {
            continue;
        }
        let Some(specifiers) = &import.specifiers else {
            continue;
        };
        let specifier = crate::ast::js_literal(&import.source);
        let Some(module) = resolve_import(specifier.as_ref(), importer, &TsConfig::default())
        else {
            continue;
        };
        let module = normalize_path(&module);
        for specifier in specifiers {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                continue;
            };
            if named.import_kind != ImportOrExportKind::Value {
                continue;
            }
            let local = named.local.name.as_str();
            if bindings.contains_key(local) {
                bindings.remove(local);
                continue;
            }
            bindings.insert(
                local.to_string(),
                Binding::Imported {
                    module: module.clone(),
                    export_name: crate::ast::module_export_name(&named.imported).into_owned(),
                },
            );
        }
    }
}

pub(super) fn exported_locals(program: &Program<'_>) -> BTreeMap<String, String> {
    let mut exports = BTreeMap::new();
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export)
                if export.export_kind == ImportOrExportKind::Value =>
            {
                for specifier in &export.specifiers {
                    if specifier.export_kind != ImportOrExportKind::Value {
                        continue;
                    }
                    let ModuleExportName::IdentifierReference(local) = &specifier.local else {
                        continue;
                    };
                    let exported = crate::ast::module_export_name(&specifier.exported);
                    exports.insert(exported.into_owned(), local.name.to_string());
                }
            }
            Statement::ExportDeclaration(export) => {
                let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                    continue;
                };
                if declaration.kind != VariableDeclarationKind::Const {
                    continue;
                }
                for declarator in &declaration.declarations {
                    if let Some(name) = binding_name(&declarator.id) {
                        exports.insert(name.to_string(), name.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    exports
}
