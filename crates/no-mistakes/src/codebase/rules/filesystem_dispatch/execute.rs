use super::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

mod special;

pub(super) type ResultAccumulator = Mutex<Vec<(&'static str, Result<Vec<RuleFinding>>)>>;

pub(super) struct RuleRunInputs<'a> {
    root: &'a Path,
    config: &'a crate::config::v2::NoMistakesConfig,
    snapshot: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    vitest_catalog: Option<&'a super::super::PreparedVitestProjectCatalog>,
    sources: &'a std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    facts: Option<&'a crate::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
    workflow_documents: Option<&'a crate::codebase::ci_workflows::ParsedWorkflowSet>,
    tsconfig_gate_project_inputs: Option<&'a tsconfig_gate_coverage::ProjectSourceInputs>,
    config_path: Option<&'a Path>,
    candidates: &'a candidate_index::RuleCandidateIndex,
    markdown_facts: &'a super::super::markdown_facts::MarkdownFactMap,
    acc: &'a ResultAccumulator,
}

/// Request-scoped inputs prepared once and shared by filesystem rules.
#[doc(hidden)]
pub struct PreparedFilesystemRuleInputs<'a> {
    pub snapshot: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    pub vitest_catalog: Option<&'a super::super::PreparedVitestProjectCatalog>,
    pub sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    pub workflow_documents: Option<&'a crate::codebase::ci_workflows::ParsedWorkflowSet>,
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
    // This legacy prepared entrypoint still owns the one request-scoped fact
    // pass for callers that did not supply one. Reuse its source store so the
    // finite-set rule can borrow call facts without reading or parsing again.
    let facts = super::entrypoints::prepare_call_site_facts(root, config, &prepared.sources);
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
        root,
        config,
        files,
        prepared,
        facts.as_ref(),
    )
}

/// Run filesystem rules using request-scoped facts prepared by the caller.
///
/// The optional lookup keeps standalone filesystem callers lightweight while
/// allowing aggregate CLI and N-API checks to share their one TS fact pass
/// with AST-backed filesystem rules.
#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    run_prepared_filesystem_rules(root, config, files, prepared, facts, false)
}

/// Aggregate check adapter that defers suppression to the shared result pass.
#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    run_prepared_filesystem_rules(root, config, files, prepared, facts, true)
}

fn run_prepared_filesystem_rules(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let PreparedFilesystemRuleInputs {
        snapshot,
        vitest_catalog,
        sources,
        workflow_documents,
        tsconfig_gate_project_inputs,
        config_path,
    } = prepared;
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
        )),
    );
    inventory::register_trusted_external_candidates(root, config, &candidates, &sources);
    let markdown_facts = markdown_dispatch::prepare(root, config, &candidates, &sources)?;
    run_enabled_rules(&RuleRunInputs {
        root,
        config,
        snapshot,
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
    for (_, result) in results {
        findings.extend(result?);
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
    super::super::sort_findings(&mut findings);
    Ok(findings)
}

fn run_enabled_rules(inputs: &RuleRunInputs<'_>) {
    macro_rules! run_rules { ($($id:expr => $call:path),* $(,)?) => { rayon::scope(|scope| { $( if rule_enabled(inputs.config, $id) { scope.spawn(|_| { let result = run_rule::run_rule_with_sources(run_rule::RunRuleRequest { rule_id: $id, fallback: $call, root: inputs.root, config: inputs.config, files: inputs.candidates.candidates($id), sources: inputs.sources, facts: inputs.facts, defer_suppression: inputs.defer_suppression }); inputs.acc.lock().expect("mutex poisoned").push(($id, result)); }); } )*; special::spawn(scope, inputs); }); }; }
    crate::filesystem_rules!(run_rules);
}
