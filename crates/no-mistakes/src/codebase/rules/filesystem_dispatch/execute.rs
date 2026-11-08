use super::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
mod allow_scope;
mod finalize;
mod next_config;
mod outcomes;
mod parallel;
mod source_demand;
mod special;
mod sql_demand;
mod timing;
pub use outcomes::{
    run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression,
    run_filesystem_rules_with_config_snapshot_catalog_sources_facts_suppression_and_session,
};
use parallel::run_enabled_rules;
pub(super) use parallel::trace_rule;
pub(super) type ResultAccumulator = Mutex<Vec<(&'static str, Result<Vec<RuleFinding>>)>>;
pub(super) struct RuleRunInputs<'a> {
    root: &'a Path,
    config: &'a crate::config::v2::NoMistakesConfig,
    snapshot: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    next_config_resolver: Option<&'a nextjs_redirect_destinations::PreparedConfigResolver<'a>>,
    vitest_catalog: Option<&'a super::super::PreparedVitestProjectCatalog>,
    sources: &'a std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    facts: Option<&'a crate::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
    workflow_documents: Option<&'a crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs>,
    tsconfig_gate_project_inputs: Option<&'a tsconfig_gate_coverage::ProjectSourceInputs>,
    config_path: Option<&'a Path>,
    candidates: &'a candidate_index::RuleCandidateIndex,
    markdown_facts: &'a super::super::markdown_facts::MarkdownFactMap,
    acc: &'a ResultAccumulator,
}
#[doc(hidden)]
pub struct PreparedFilesystemRuleInputs<'a> {
    pub session: Option<&'a crate::codebase::analysis_session::AnalysisSession>,
    pub next_config_resolver: Option<&'a nextjs_redirect_destinations::PreparedConfigResolver<'a>>,
    pub snapshot: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    pub vitest_catalog: Option<&'a super::super::PreparedVitestProjectCatalog>,
    pub sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    pub workflow_documents:
        Option<&'a crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs>,
    pub tsconfig_gate_project_inputs: Option<&'a tsconfig_gate_coverage::ProjectSourceInputs>,
    pub config_path: Option<&'a Path>,
}
#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_and_sources(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
) -> Result<Vec<RuleFinding>> {
    let facts =
        super::entrypoints::prepare_call_site_facts(root, config, files, &prepared.sources)?;
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
        root,
        config,
        files,
        prepared,
        facts.as_ref(),
    )
}
#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    let owned_facts = if facts.is_none()
        && (rule_enabled(config, FORBIDDEN_LINT_SUPPRESSIONS)
            || rule_enabled(config, PRODUCTION_DEPENDENCY_DECLARATIONS)
            || rule_enabled(config, REQUIRED_COMPANION_IMPORTS)
            || crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS
                .iter()
                .any(|rule_id| rule_enabled(config, rule_id)))
    {
        super::entrypoints::prepare_call_site_facts(root, config, files, &prepared.sources)?
    } else {
        None
    };
    next_config::run(
        root,
        config,
        files,
        prepared,
        facts.or(owned_facts.as_ref()),
        false,
    )
}

fn run_prepared_filesystem_rules(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let PreparedFilesystemRuleInputs {
        session: _,
        snapshot,
        next_config_resolver,
        vitest_catalog,
        sources,
        workflow_documents,
        tsconfig_gate_project_inputs,
        config_path,
    } = prepared;
    let scoped = allow_scope::config(root, config, files, snapshot, &sources)?;
    let config = scoped.as_ref();
    let acc = Mutex::new(Vec::new());
    let metadata_files = metadata::metadata_files(root, config, files, snapshot);
    let candidates = candidate_index::RuleCandidateIndex::prepare_with_inventory(
        root,
        config,
        files,
        &snapshot.tracked_paths_from(files),
        &metadata_files,
        Some(inventory::tracked_inventory_with_markdown_project_roots(
            root, config, snapshot,
        )?),
    )?;
    inventory::register_trusted_external_candidates(root, config, &candidates, &sources)?;
    let markdown_facts = markdown_dispatch::prepare(root, config, &candidates, &sources)?;
    run_enabled_rules(&RuleRunInputs {
        root,
        config,
        snapshot,
        next_config_resolver,
        vitest_catalog,
        sources: &sources,
        facts,
        defer_suppression,
        workflow_documents,
        tsconfig_gate_project_inputs,
        config_path,
        candidates: &candidates,
        markdown_facts: &markdown_facts,
        acc: &acc,
    });
    let mut results = acc.into_inner().expect("mutex poisoned");
    results.sort_unstable_by_key(|(id, _)| *id);
    let mut findings = Vec::new();
    for (id, result) in results {
        if let Some(session) = session {
            session.capture_check_result(
                id,
                outcomes::register_demand(
                    root,
                    config,
                    id,
                    candidates.candidates(id),
                    facts,
                    &sources,
                    session,
                ),
            )?;
            findings.extend(
                session
                    .capture_check_result(id, result)?
                    .unwrap_or_default(),
            );
        } else {
            findings.extend(result?);
        }
    }
    if let Some(facts) = facts {
        findings.extend(
            super::super::postgres_unanalyzable_schema::diagnostics(root, config, files, facts)?
                .findings,
        );
    }
    if !defer_suppression {
        suppress_rule_findings_with_sources_except(
            root,
            &mut findings,
            &sources,
            &[
                RUST_MAX_LINES_PER_FILE,
                RUST_NO_INLINE_TESTS,
                RUST_NO_INLINE_ALLOWS,
            ],
        );
    }
    finalize::findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
#[path = "execute/tests.rs"]
mod tests;
