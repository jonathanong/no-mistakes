use super::super::shared::rel_str;
use crate::codebase::dependencies::graph::SymbolIndex;
use crate::codebase::ts_symbols::{Export, ExportKind};
use std::path::Path;

/// The symbol name a concrete export is indexed under. Default exports are
/// recorded under `default` regardless of the local declaration name.
pub(crate) fn export_lookup_symbol(export: &Export) -> String {
    match export.kind {
        ExportKind::Default => "default".to_string(),
        _ => export.name.clone(),
    }
}

/// True for an anonymous `export * from '...'` row, whose consumers import
/// concrete names from the re-exporting file rather than a single symbol. A
/// named `export * as ns from '...'` has a concrete public name (`ns`) and is
/// not a transparent star row.
fn is_star_reexport(export: &Export) -> bool {
    export.name == "*"
        && matches!(&export.kind, ExportKind::ReExport { imported, .. } if imported == "*")
}

fn dedup_sorted(mut paths: Vec<String>) -> Vec<String> {
    paths.sort();
    paths.dedup();
    paths
}

fn symbol_importers(index: &SymbolIndex, file: &Path, symbol: &str, root: &Path) -> Vec<String> {
    index
        .importers_of(file, symbol)
        .map(|records| {
            records
                .iter()
                .map(|(i, _, _)| rel_str(i.as_ref(), root))
                .collect()
        })
        .unwrap_or_default()
}

/// Importers recorded under the wildcard `*` — namespace imports
/// (`import * as ns`), namespace re-exports (`export * as ns`, recorded with a
/// concrete local name), and anonymous `export *` star barrels (local name
/// `*`). When `exclude_anon_star` is set (for the `default` export, which an
/// anonymous `export *` does not forward) only the anonymous star rows are
/// dropped; namespace imports and `export * as ns` still expose `.default`.
fn wildcard_importers(
    index: &SymbolIndex,
    file: &Path,
    root: &Path,
    exclude_anon_star: bool,
) -> Vec<String> {
    index
        .importers_of(file, "*")
        .map(|records| {
            records
                .iter()
                .filter(|(_, local, is_reexport)| {
                    !(exclude_anon_star && *is_reexport && local.as_ref() == "*")
                })
                .map(|(i, _, _)| rel_str(i.as_ref(), root))
                .collect()
        })
        .unwrap_or_default()
}

/// Unique importer files that reference `(file, symbol)`, root-relative and
/// sorted. Includes barrel re-exporters and wildcard importers (namespace
/// imports always; `export *` barrels for every symbol except `default`).
fn importer_paths(index: &SymbolIndex, file: &Path, symbol: &str, root: &Path) -> Vec<String> {
    let mut paths = symbol_importers(index, file, symbol, root);
    if symbol != "*" {
        paths.extend(wildcard_importers(index, file, root, symbol == "default"));
    }
    dedup_sorted(paths)
}

/// Importers recorded for exactly `(file, symbol)`, with no wildcard widening.
/// Used for names that are not (or no longer) exports, where a namespace import
/// or `export *` barrel does not reference the specific deleted name.
pub(crate) fn direct_importer_paths(
    index: &SymbolIndex,
    file: &Path,
    symbol: &str,
    root: &Path,
) -> Vec<String> {
    dedup_sorted(symbol_importers(index, file, symbol, root))
}

/// All importers of any symbol of `file` — the consumers of an `export *` row,
/// who import concrete names rather than the star itself.
pub(super) fn file_importer_paths(index: &SymbolIndex, file: &Path, root: &Path) -> Vec<String> {
    dedup_sorted(
        index
            .file_importers(file)
            .iter()
            .map(|path| rel_str(path, root))
            .collect(),
    )
}

/// Importers of a specific export row. `export *` rows resolve to their
/// concrete-name consumers; every other row uses the symbol lookup.
pub(crate) fn export_importer_paths(
    index: &SymbolIndex,
    file: &Path,
    export: &Export,
    root: &Path,
) -> Vec<String> {
    if is_star_reexport(export) {
        file_importer_paths(index, file, root)
    } else {
        importer_paths(index, file, &export_lookup_symbol(export), root)
    }
}
