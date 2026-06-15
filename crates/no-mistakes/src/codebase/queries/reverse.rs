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

fn symbol_importers(
    index: &SymbolIndex,
    abs_file: &Path,
    symbol: &str,
    root: &Path,
) -> Vec<String> {
    index
        .importers_of(abs_file, symbol)
        .map(|records| {
            records
                .iter()
                .map(|(importer, _, _)| rel_str(importer, root))
                .collect()
        })
        .unwrap_or_default()
}

/// Unique importer files of `(abs_file, symbol)`, as sorted root-relative
/// strings. Includes barrel re-exporters and — because namespace imports
/// (`import * as ns`) and star re-exports (`export *`) are indexed under the
/// wildcard `*` and reference every concrete export — the wildcard importers
/// too.
pub(crate) fn importer_paths(
    index: &SymbolIndex,
    abs_file: &Path,
    symbol: &str,
    root: &Path,
) -> Vec<String> {
    let mut paths = symbol_importers(index, abs_file, symbol, root);
    if symbol != "*" {
        paths.extend(symbol_importers(index, abs_file, "*", root));
    }
    paths.sort();
    paths.dedup();
    paths
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
