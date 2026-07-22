//! `symbols` binary: dump the named exports and imports of one or more TS/JS files.
//!
//! Wraps `crate::codebase::ts_symbols::extract_symbols_at_path` and renders the result as JSON
//! (default for non-TTY), YAML, Markdown, paths, or a human tree.
//!
//! Re-export `source` and import `source` specifiers are resolved through
//! `crate::codebase::ts_resolver` to project-relative paths when possible, so an agent
//! can follow the chain without a second tool invocation.

pub mod output;

use anyhow::{Context, Result};
use clap::Parser;
use is_terminal::IsTerminal;
use rayon::prelude::*;
use serde::Serialize;
use std::io;
use std::path::{Path, PathBuf};

pub use crate::codebase::dependencies::Format;
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_symbols::{Export, ExportKind, FileSymbols, NamedImport};

include!("types.rs");
include!("resolve.rs");
include!("pipeline.rs");
include!("pipeline_output.rs");
include!("entry.rs");
include!("filters.rs");

mod impact {
    use super::*;
    include!("impact.rs");
}

pub(crate) fn signature_impact_graph_plan() -> crate::codebase::dependencies::graph::GraphBuildPlan
{
    impact::signature_impact_graph_plan()
}

pub(crate) struct PreparedSignatureImpact<'a> {
    pub(crate) session: &'a crate::codebase::analysis_session::AnalysisSession,
    pub(crate) tsconfig_catalog: &'a crate::codebase::ts_resolver::TsConfigCatalog,
    pub(crate) graph_files: &'a crate::codebase::dependencies::graph::GraphFiles,
    pub(crate) test_filter: &'a crate::codebase::test_filter::TestFileFilter,
    pub(crate) workspace: &'a crate::codebase::workspaces::IndexedWorkspaceMap,
    pub(crate) graph: &'a crate::codebase::dependencies::graph::DepGraph,
    pub(crate) facts: &'a crate::codebase::ts_source::facts::TsFactMap,
}

pub(crate) fn signature_impact_json_with_prepared(
    args: &SymbolsArgs,
    root: &Path,
    prepared: PreparedSignatureImpact<'_>,
) -> Result<String> {
    let report = impact::collect_report_with_prepared(args, root, prepared)?;
    let mut output = Vec::new();
    impact::write_report(&report, Format::Json, &mut output)?;
    String::from_utf8(output).context("signature-impact JSON output must be UTF-8")
}

#[cfg(test)]
mod tests;
