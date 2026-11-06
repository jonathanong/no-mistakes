mod config;
mod extends;
mod nextjs;
mod prerequisites;
mod script_entries;
mod selector_advisories;
mod selector_gaps;
mod traversal;
mod used_in_file;

use super::analysis_policy::AnalysisDiagnostics;
use super::path_filter::RulePathFilter;
use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::ts_symbols::{ExportKind, FileSymbols};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

pub const RULE_ID: &str = "production-exports-only-used-by-tests";

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| GraphBuildPlan {
        symbol_import_selectors: true,
        extends: true,
        ..Default::default()
    })
}

mod execution;
pub(crate) use execution::check_with_graph_and_session;

fn forwarded(symbols: &FileSymbols, export: &crate::codebase::ts_symbols::Export) -> bool {
    matches!(export.kind, ExportKind::ReExport { .. })
        || symbols
            .imports
            .iter()
            .any(|import| import.local == export.local.as_deref().unwrap_or(&export.name))
}

fn finding(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    export: &crate::codebase::ts_symbols::Export,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: relative_slash_path(root, file),
        line: export.line.max(1) as usize,
        message: application.message.as_ref().map_or_else(|| format!(
            "production export `{}` is imported only by configured tests or test helpers; move it into a configured test-helper root or remove its export. Production APIs should serve production code",
            export.name,
        ), |message| format!("{message}: `{}`", export.name)),
        import: None,
        target: Some(export.name.clone()),
    }
}

fn uncertain(
    root: &Path,
    application: &RuleDef,
    file: &Path,
    line: usize,
    reason: &str,
) -> RuleFinding {
    let detail = format!("cannot analyze production export importers: {reason}; use explicit named imports and statically analyzable exports");
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: relative_slash_path(root, file),
        line: line.max(1),
        message: application
            .message
            .as_ref()
            .map_or_else(|| detail.clone(), |message| format!("{message}: {detail}")),
        import: None,
        target: Some("unanalyzable".to_owned()),
    }
}

#[cfg(test)]
mod nextjs_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod work_tests;
#[cfg(test)]
mod workspace_exports_tests;
