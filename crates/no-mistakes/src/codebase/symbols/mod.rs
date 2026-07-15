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
use crate::codebase::ts_symbols::{
    extract_symbols_at_path, Export, ExportKind, FileSymbols, NamedImport,
};

include!("types.rs");
include!("resolve.rs");
include!("pipeline.rs");
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

pub(crate) fn signature_impact_json_with_prepared(
    args: &SymbolsArgs,
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    graph_files: &crate::codebase::dependencies::graph::GraphFiles,
    test_filter: &crate::codebase::test_filter::TestFileFilter,
    graph: &crate::codebase::dependencies::graph::DepGraph,
    facts: &crate::codebase::ts_source::facts::TsFactMap,
) -> Result<String> {
    let report = impact::collect_report_with_prepared(
        args,
        root,
        tsconfig,
        graph_files,
        test_filter,
        graph,
        facts,
    )?;
    let mut output = Vec::new();
    impact::write_report(&report, Format::Json, &mut output)?;
    String::from_utf8(output).context("signature-impact JSON output must be UTF-8")
}

#[cfg(test)]
mod tests;
