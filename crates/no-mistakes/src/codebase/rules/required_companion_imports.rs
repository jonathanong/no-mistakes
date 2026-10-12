mod helpers;

mod plan_merge;
mod prepared;
mod projection;
pub(crate) use plan_merge::merge_plans;
#[doc(hidden)]
pub use prepared::{configure_fact_plan, PreparedCompanionImportPlan};

use super::RuleFinding;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const RULE_ID: &str = "required-companion-imports";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) source_dirs: Vec<String>,
    pub(crate) source_globs: Vec<String>,
    pub(crate) source_extensions: Vec<String>,
    pub(crate) direct_child_only: bool,
    pub(crate) exclude_basenames: Vec<String>,
    pub(crate) exclude_prefixes: Vec<String>,
    pub(crate) companion_globs: Vec<String>,
    pub(crate) specifier_template: String,
    pub(crate) strip_source_prefix: String,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let mut plan = crate::codebase::check_facts::CheckFactPlan::default();
    configure_fact_plan(root, config, all_files, sources, &mut plan)?;
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session, root, (Vec::new(), Vec::new()), plan, None, Arc::clone(sources),
    );
    check_with_files_sources_and_facts(root, config, all_files, sources, Some(&facts))
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &Arc<crate::codebase::ts_source::SourceStore>,
    facts: Option<&dyn TsFactLookup>,
) -> Result<Vec<RuleFinding>> {
    let Some(facts) = facts else {
        return check_with_files_and_sources(root, config, all_files, sources);
    };
    if config.rule_applications(RULE_ID).is_empty() {
        return Ok(Vec::new());
    }
    let prepared = facts
        .companion_import_plans()
        .iter()
        .find(|plan| plan.matches(root, config))
        .ok_or_else(|| {
            anyhow::anyhow!("required-companion-imports requires its prepared companion selection")
        })?;
    projection::findings(prepared, facts, sources)
}

#[cfg(test)]
mod tests;
