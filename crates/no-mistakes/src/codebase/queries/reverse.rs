use super::shared::Target;
use crate::codebase::dependencies::graph::SymbolIndex;
use crate::codebase::ts_symbols::{Export, ExportKind, FileSymbols};
use anyhow::Result;
use std::path::Path;

mod build;
mod importers;
pub(crate) use build::{
    build_reverse_analysis, build_reverse_analysis_with_plan, build_reverse_index_from_prepared,
};
pub(crate) use importers::{direct_importer_paths, export_importer_paths, export_lookup_symbol};

pub(crate) struct ReverseAnalysis {
    pub(crate) index: SymbolIndex,
    facts: crate::codebase::ts_source::facts::TsFactMap,
    target_tsconfig: crate::codebase::ts_resolver::TsConfig,
}

impl ReverseAnalysis {
    pub(crate) fn facts_at(
        &self,
        path: &Path,
    ) -> Option<&crate::codebase::ts_source::facts::TsFileFacts> {
        self.facts.get(path)
    }

    pub(crate) fn target_tsconfig(&self) -> &crate::codebase::ts_resolver::TsConfig {
        &self.target_tsconfig
    }

    pub(crate) fn symbols(&self, target: &Target) -> Result<FileSymbols> {
        let Some(facts) = self.facts.get(&target.abs_file) else {
            anyhow::bail!("missing facts for {}", target.abs_file.display());
        };
        if facts.fatal_parse_error {
            if let Some(error) = &facts.parse_error {
                anyhow::bail!(
                    "extracting symbols from {}: {error}",
                    target.abs_file.display()
                );
            }
            anyhow::bail!(
                "fatal parser failure while extracting symbols from {}",
                target.abs_file.display()
            );
        }
        // Recovered parser facts can retain top-level symbols alongside a
        // diagnostic. Direct symbol extraction does the same unless the
        // parser panicked, which is tracked separately above.
        if let Some(symbols) = facts.symbols.clone() {
            return Ok(symbols);
        }
        if let Some(error) = &facts.parse_error {
            anyhow::bail!(
                "extracting symbols from {}: {error}",
                target.abs_file.display()
            );
        }
        anyhow::bail!("missing symbols for {}", target.abs_file.display());
    }
}

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

#[cfg(test)]
mod tests;
