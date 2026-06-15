use super::shared::{rel_str, Target};
use crate::codebase::dependencies::graph::SymbolIndex;
use crate::codebase::ts_symbols::{Export, ExportKind};
use anyhow::Result;
use std::path::Path;

/// Build the reverse import index for the whole project in one parallel scan.
/// Cheaper than a full `DepGraph` — it only resolves import/re-export edges.
pub(crate) fn build_index(target: &Target) -> Result<SymbolIndex> {
    SymbolIndex::build_from_root(&target.root, &target.tsconfig)
}

/// The symbol name a concrete export is indexed under. Default exports are
/// recorded under `default` regardless of the local declaration name.
pub(crate) fn export_lookup_symbol(export: &Export) -> String {
    match export.kind {
        ExportKind::Default => "default".to_string(),
        _ => export.name.clone(),
    }
}

/// True for an `export * from '...'` row, whose consumers import concrete names
/// from the re-exporting file rather than a single symbol.
fn is_star_reexport(export: &Export) -> bool {
    matches!(&export.kind, ExportKind::ReExport { imported, .. } if imported == "*")
}

fn dedup_sorted(mut paths: Vec<String>) -> Vec<String> {
    paths.sort();
    paths.dedup();
    paths
}

fn symbol_importers(index: &SymbolIndex, file: &Path, symbol: &str, root: &Path) -> Vec<String> {
    index
        .importers_of(file, symbol)
        .map(|records| records.iter().map(|(i, _, _)| rel_str(i, root)).collect())
        .unwrap_or_default()
}

/// Importers recorded under the wildcard `*` — namespace imports
/// (`import * as ns`) and `export *` star barrels. `include_reexport=false`
/// keeps only namespace imports, used for the `default` export which `export *`
/// does not forward.
fn wildcard_importers(
    index: &SymbolIndex,
    file: &Path,
    root: &Path,
    include_reexport: bool,
) -> Vec<String> {
    index
        .importers_of(file, "*")
        .map(|records| {
            records
                .iter()
                .filter(|(_, _, is_reexport)| include_reexport || !*is_reexport)
                .map(|(i, _, _)| rel_str(i, root))
                .collect()
        })
        .unwrap_or_default()
}

/// Unique importer files that reference `(file, symbol)`, root-relative and
/// sorted. Includes barrel re-exporters and wildcard importers (namespace
/// imports always; `export *` barrels for every symbol except `default`).
pub(crate) fn importer_paths(
    index: &SymbolIndex,
    file: &Path,
    symbol: &str,
    root: &Path,
) -> Vec<String> {
    let mut paths = symbol_importers(index, file, symbol, root);
    if symbol != "*" {
        paths.extend(wildcard_importers(index, file, root, symbol != "default"));
    }
    dedup_sorted(paths)
}

/// All importers of any symbol of `file` — the consumers of an `export *` row,
/// who import concrete names rather than the star itself.
pub(crate) fn file_importer_paths(index: &SymbolIndex, file: &Path, root: &Path) -> Vec<String> {
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

pub(crate) fn export_kind_str(kind: &ExportKind) -> &'static str {
    match kind {
        ExportKind::Function => "function",
        ExportKind::Class => "class",
        ExportKind::Const => "const",
        ExportKind::Let => "let",
        ExportKind::Var => "var",
        ExportKind::TypeAlias => "type",
        ExportKind::Interface => "interface",
        ExportKind::Enum => "enum",
        ExportKind::Default => "default",
        ExportKind::ReExport { .. } => "re-export",
    }
}
