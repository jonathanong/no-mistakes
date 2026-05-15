use oxc_ast::ast::{ExportNamedDeclaration, ImportDeclarationSpecifier, ImportOrExportKind};
use oxc_span::GetSpan;

pub(crate) fn is_runtime_import(import: &oxc_ast::ast::ImportDeclaration) -> bool {
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

pub(crate) fn is_runtime_export(export: &ExportNamedDeclaration, source: &str) -> bool {
    if export.export_kind == ImportOrExportKind::Type {
        return false;
    }

    let raw = declaration_text(
        export.span().start as usize,
        export.span().end as usize,
        source,
    );

    match parse_named_specifiers(raw) {
        Some(named_specifiers) => {
            if named_specifiers.is_empty() {
                return true;
            }
            named_specifiers
                .iter()
                .any(|specifier| !specifier.trim_start().starts_with("type "))
        }
        None => true,
    }
}

pub(crate) fn declaration_text(start: usize, end: usize, source: &str) -> &str {
    if start > end || end > source.len() {
        return "";
    }
    &source[start..end]
}

pub(crate) fn parse_named_specifiers(statement: &str) -> Option<Vec<&str>> {
    let start = statement.find('{')?;
    let end = statement.rfind('}')?;
    if end <= start {
        return Some(Vec::new());
    }
    let names = statement[start + 1..end]
        .split(',')
        .map(|segment| segment.trim())
        .filter(|segment| !segment.is_empty())
        .collect();
    Some(names)
}
