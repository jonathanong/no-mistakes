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
    workflow_documents: Option<&'a crate::codebase::ci_workflows::ParsedWorkflowSet>,
    config_path: Option<&'a Path>,
    candidates: &'a candidate_index::RuleCandidateIndex,
    acc: &'a ResultAccumulator,
}

/// Request-scoped inputs prepared once and shared by filesystem rules.
#[doc(hidden)]
pub struct PreparedFilesystemRuleInputs<'a> {
    pub snapshot: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    pub vitest_catalog: Option<&'a super::super::PreparedVitestProjectCatalog>,
    pub sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    pub workflow_documents: Option<&'a crate::codebase::ci_workflows::ParsedWorkflowSet>,
    pub config_path: Option<&'a Path>,
}

#[doc(hidden)]
pub fn run_filesystem_rules_with_config_snapshot_catalog_and_sources(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
) -> Result<Vec<RuleFinding>> {
    let PreparedFilesystemRuleInputs {
        snapshot,
        vitest_catalog,
        sources,
        workflow_documents,
        config_path,
    } = prepared;
    let acc = Mutex::new(Vec::new());
    let metadata_files = metadata_files(root, config, files, snapshot);
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
    run_enabled_rules(&RuleRunInputs {
        root,
        config,
        snapshot,
        vitest_catalog,
        sources: &sources,
        workflow_documents,
        config_path,
        candidates: &candidates,
        acc: &acc,
    });
    let mut results = acc.into_inner().expect("mutex poisoned");
    results.sort_unstable_by_key(|(id, _)| *id);
    let mut findings = Vec::new();
    for (_, result) in results {
        findings.extend(result?);
    }
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
    super::super::sort_findings(&mut findings);
    Ok(findings)
}

fn metadata_files(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
) -> Vec<PathBuf> {
    if !rule_enabled(config, FORBIDDEN_WORKSPACE_CLOSURE)
        && !rule_enabled(config, PRODUCTION_DEPENDENCY_DECLARATIONS)
    {
        return Vec::new();
    }
    let mut metadata_files = files.to_vec();
    metadata_files.extend(snapshot.paths_for(root).iter().cloned());
    metadata_files.sort();
    metadata_files.dedup();
    metadata_files
}

fn run_enabled_rules(inputs: &RuleRunInputs<'_>) {
    macro_rules! run_rules { ($($id:expr => $call:path),* $(,)?) => { rayon::scope(|scope| { $( if rule_enabled(inputs.config, $id) { scope.spawn(|_| { let result = run_rule::run_rule_with_sources($id, $call, inputs.root, inputs.config, inputs.candidates.candidates($id), inputs.sources); inputs.acc.lock().expect("mutex poisoned").push(($id, result)); }); } )*; spawn_special_rules(scope, inputs); }); }; }
    crate::filesystem_rules!(run_rules);
}

fn spawn_special_rules<'a>(scope: &rayon::Scope<'a>, inputs: &'a RuleRunInputs<'a>) {
    let RuleRunInputs {
        root,
        config,
        snapshot,
        vitest_catalog,
        sources,
        workflow_documents,
        config_path,
        candidates,
        acc,
    } = *inputs;
    if rule_enabled(config, MARKDOWN_REACHABILITY) {
        scope.spawn(|_| {
            let result = markdown_reachability::check_with_files_and_sources(
                root,
                config,
                candidates.candidates(MARKDOWN_REACHABILITY),
                sources,
            );
            acc.lock()
                .expect("mutex poisoned")
                .push((MARKDOWN_REACHABILITY, result));
        });
    }
    if rule_enabled(config, MARKDOWN_STRUCTURE_BUDGET) {
        scope.spawn(|_| {
            let result = markdown_structure_budget::check_with_files_and_sources(
                root,
                config,
                candidates.candidates(MARKDOWN_STRUCTURE_BUDGET),
                sources,
            );
            acc.lock()
                .expect("mutex poisoned")
                .push((MARKDOWN_STRUCTURE_BUDGET, result));
        });
    }
    if registry::rust_rules_enabled(config) {
        scope.spawn(|_| {
            let result = rust_rules_combined::check_with_files_and_sources(
                root,
                config,
                candidates.rust_candidates(),
                candidates.exclusive_rust_candidates(),
                sources,
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
            let result = workflow_documents.map_or_else(
                || {
                    Err(anyhow::anyhow!(
                        "prepared workflow documents are required for {TSCONFIG_GATE_COVERAGE}"
                    ))
                },
                |workflows| {
                    tsconfig_gate_coverage::check_with_prepared(
                        root,
                        config,
                        tsconfig_gate_coverage::PreparedInputs {
                            tracked_paths: snapshot.tracked_paths_for(root).as_ref(),
                            workflows,
                            _sources: sources,
                            config_path,
                        },
                    )
                },
            );
            acc.lock()
                .expect("mutex poisoned")
                .push((TSCONFIG_GATE_COVERAGE, result));
        });
    }
}
