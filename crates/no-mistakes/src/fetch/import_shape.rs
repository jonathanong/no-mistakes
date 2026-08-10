use oxc_ast::ast::{ExportFromDeclaration, ImportDeclarationSpecifier, ImportOrExportKind};

pub fn is_runtime_import(import: &oxc_ast::ast::ImportDeclaration) -> bool {
    if import.import_kind == ImportOrExportKind::Type {
        return false;
    }

    let Some(specifiers) = &import.specifiers else {
        return true;
    };
    if specifiers.is_empty() {
        return true;
    }

    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => return true,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => return true,
            ImportDeclarationSpecifier::ImportSpecifier(import_specifier) => {
                if import_specifier.import_kind == ImportOrExportKind::Value {
                    return true;
                }
            }
        }
    }

    false
}

pub fn is_runtime_export(export: &ExportFromDeclaration) -> bool {
    if export.export_kind == ImportOrExportKind::Type {
        return false;
    }

    if export.specifiers.is_empty() {
        return true;
    }
    export
        .specifiers
        .iter()
        .any(|spec| spec.export_kind == ImportOrExportKind::Value)
}
