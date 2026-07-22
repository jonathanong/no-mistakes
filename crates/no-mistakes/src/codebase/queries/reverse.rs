use super::shared::{rel_str, Target};
use crate::codebase::dependencies::graph::SymbolIndex;
use crate::codebase::ts_symbols::{Export, ExportKind, FileSymbols};
use anyhow::Result;
use std::path::Path;

/// Find an export by its public name, also accepting `default` for the default
/// export (whose stored name is the local declaration name).
pub(crate) fn find_export<'a>(symbols: &'a FileSymbols, name: &str) -> Option<&'a Export> {
    symbols
        .exports
        .iter()
        .find(|export| export.name == name)
        .or_else(|| {
            (name == "default")
                .then(|| {
                    symbols
                        .exports
                        .iter()
                        .find(|export| export.kind == ExportKind::Default)
                })
                .flatten()
        })
}

/// Build the reverse import index for the whole project in one parallel scan.
/// Cheaper than a full `DepGraph` — it only resolves import/re-export edges.
pub(crate) fn build_index(target: &Target) -> Result<SymbolIndex> {
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::discover(&target.root);
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        graph_files.indexable(),
        crate::codebase::ts_source::facts::TsFactPlan::imports_and_symbols(),
    );
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let workspace =
        crate::codebase::workspaces::load_indexed_from_files(&target.root, graph_files.all())
            .unwrap_or_default();
    Ok(
        SymbolIndex::build_from_facts_workspace_resolution_cache_and_session(
            &target.tsconfig,
            Some(&target.tsconfig_catalog),
            &graph_files,
            &facts,
            &workspace,
            None,
            &session,
        ),
    )
}

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
        .map(|records| records.iter().map(|(i, _, _)| rel_str(i, root)).collect())
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
                    !(exclude_anon_star && *is_reexport && local == "*")
                })
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
