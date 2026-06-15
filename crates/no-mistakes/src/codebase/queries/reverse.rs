use super::shared::{rel_str, Target};
use crate::codebase::dependencies::graph::SymbolIndex;
use crate::codebase::ts_symbols::ExportKind;
use anyhow::Result;
use std::path::Path;

/// Build the reverse import index for the whole project in one parallel scan.
/// Cheaper than a full `DepGraph` — it only resolves import/re-export edges.
pub(crate) fn build_index(target: &Target) -> Result<SymbolIndex> {
    SymbolIndex::build_from_root(&target.root, &target.tsconfig)
}

/// Unique importer files of `(abs_file, symbol)`, as sorted root-relative
/// strings. Includes barrel re-exporters (they reference the symbol).
pub(crate) fn importer_paths(
    index: &SymbolIndex,
    abs_file: &Path,
    symbol: &str,
    root: &Path,
) -> Vec<String> {
    let mut paths: Vec<String> = index
        .importers_of(abs_file, symbol)
        .map(|records| {
            records
                .iter()
                .map(|(importer, _, _)| rel_str(importer, root))
                .collect()
        })
        .unwrap_or_default();
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
