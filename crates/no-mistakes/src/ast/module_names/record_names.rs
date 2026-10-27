//! Fatal parses retain module names even after discarding the program AST.
use std::borrow::Cow;

use oxc_span::Span;
use oxc_str::JSStr;
use oxc_syntax::module_record::{
    ExportExportName, ExportImportName, ExportLocalName, ImportImportName, ModuleRecord,
};

pub(super) fn first_legacy_name(record: &ModuleRecord<'_>, source: &str) -> Option<Span> {
    let imports = record
        .import_entries
        .iter()
        .filter_map(|entry| match &entry.import_name {
            ImportImportName::Name(name) => legacy_span(name.name.as_str(), name.span, source),
            _ => None,
        });
    let exports = record
        .local_export_entries
        .iter()
        .chain(&record.indirect_export_entries)
        .chain(&record.star_export_entries)
        .flat_map(|entry| {
            let imported = match &entry.import_name {
                ExportImportName::Name(name) => Some(name),
                _ => None,
            };
            let exported = match &entry.export_name {
                ExportExportName::Name(name) => Some(name),
                _ => None,
            };
            let local = match &entry.local_name {
                ExportLocalName::Name(name) => Some(name),
                _ => None,
            };
            [imported, exported, local].into_iter().flatten()
        })
        .filter_map(|name| legacy_span(name.name.as_str(), name.span, source));
    let bindings = record
        .exported_bindings
        .iter()
        .filter_map(|(name, span)| legacy_span(name.as_str(), *span, source));
    imports
        .chain(exports)
        .chain(bindings)
        .min_by_key(|span| span.start)
}

fn legacy_span(value: &str, span: Span, source: &str) -> Option<Span> {
    let raw = source.get(span.start as usize..span.end as usize)?;
    // Identifier names and dummy recovery spans cannot be quoted module names.
    if !raw.starts_with(['\'', '"']) {
        return None;
    }
    matches!(
        super::super::js_string::js_raw_literal(JSStr::from(value), Some(raw)),
        Cow::Owned(_)
    )
    .then_some(span)
}
