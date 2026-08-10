use super::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

type ResultAccumulator = Mutex<Vec<(&'static str, Result<Vec<RuleFinding>>)>>;

struct RuleRunInputs<'a> {
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
    /// Aggregate `check` applies SourceStore-backed suppression once after all
    /// domains finish so it can report optional directive accounting.
    pub defer_suppression: bool,
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
    let PreparedFilesystemRuleInputs {
        snapshot,
        vitest_catalog,
        sources,
        workflow_documents,
        tsconfig_gate_project_inputs,
        config_path,
        defer_suppression,
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
    macro_rules! run_rules { ($($id:expr => $call:path),* $(,)?) => { rayon::scope(|scope| { $( if rule_enabled(inputs.config, $id) { scope.spawn(|_| { let result = run_rule::run_rule_with_sources(run_rule::RunRuleRequest { rule_id: $id, fallback: $call, root: inputs.root, config: inputs.config, files: inputs.candidates.candidates($id), sources: inputs.sources, facts: inputs.facts, defer_suppression: inputs.defer_suppression }); inputs.acc.lock().expect("mutex poisoned").push(($id, result)); }); } )*; spawn_special_rules(scope, inputs); }); }; }
    crate::filesystem_rules!(run_rules);
}

fn spawn_special_rules<'a>(scope: &rayon::Scope<'a>, inputs: &'a RuleRunInputs<'a>) {
    let RuleRunInputs {
        root,
        config,
        snapshot,
        vitest_catalog,
        sources,
        facts: _,
        defer_suppression,
        workflow_documents,
        tsconfig_gate_project_inputs,
        config_path,
        candidates,
        markdown_facts,
        acc,
    } = *inputs;
    markdown_dispatch::spawn(scope, root, config, candidates, markdown_facts, acc);
    if registry::rust_rules_enabled(config) {
        scope.spawn(move |_| {
            let result = rust_rules_combined::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                candidates.rust_candidates(),
                sources,
                defer_suppression,
            );
            acc.lock()
                .expect("mutex poisoned")
                .push(("rust-rules-combined", result));
        });
    }
    if rule_enabled(config, VITEST_PROJECT_MAPPING) {
        scope.spawn(move |_| {
            let result = vitest_project_mapping::check_with_files_and_catalog(
                root,
                config,
                candidates.candidates(VITEST_PROJECT_MAPPING),
                vitest_catalog,
            );
            acc.lock()
                .expect("mutex poisoned")
                .push((VITEST_PROJECT_MAPPING, result));
        });
    }
    if rule_enabled(config, VITEST_CI_PATH_COVERAGE) {
        scope.spawn(move |_| { let result = vitest_ci_path_coverage::check_with_files_from_snapshot_catalog_sources_and_workflows(root, config, candidates.candidates(VITEST_CI_PATH_COVERAGE), snapshot, vitest_catalog, sources, workflow_documents); acc.lock().expect("mutex poisoned").push((VITEST_CI_PATH_COVERAGE, result)); });
    }
    if rule_enabled(config, TSCONFIG_GATE_COVERAGE) {
        scope.spawn(move |_| {
            let result = workflow_documents
                .zip(tsconfig_gate_project_inputs)
                .map_or_else(
                || {
                    Err(anyhow::anyhow!(
                        "prepared workflow documents and project inputs are required for {TSCONFIG_GATE_COVERAGE}"
                    ))
                },
                |(workflows, project_source_inputs)| {
                    tsconfig_gate_coverage::check_with_prepared(
                        root,
                        config,
                        tsconfig_gate_coverage::PreparedInputs {
                            tracked_paths: snapshot.tracked_paths_for(root).as_ref(),
                            workflows,
                            project_source_inputs,
                            sources,
                            config_path,
                        },
                    )
                });
            acc.lock()
                .expect("mutex poisoned")
                .push((TSCONFIG_GATE_COVERAGE, result));
        });
    }
}
