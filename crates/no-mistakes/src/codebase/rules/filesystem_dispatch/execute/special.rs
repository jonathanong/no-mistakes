use super::*;

pub(super) fn spawn<'a>(scope: &rayon::Scope<'a>, inputs: &'a RuleRunInputs<'a>) {
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
    markdown_dispatch::spawn(
        scope,
        root,
        config,
        candidates,
        markdown_facts,
        sources,
        acc,
    );
    if registry::rust_rules_enabled(config) {
        scope.spawn(move |_| {
            let result = super::trace_rule(sources, "rust-rules-combined", || {
                rust_rules_combined::check_with_files_sources_and_deferred_suppression(
                    root,
                    config,
                    candidates.rust_candidates(),
                    sources,
                    defer_suppression,
                )
            });
            acc.lock()
                .expect("mutex poisoned")
                .push(("rust-rules-combined", result));
        });
    }
    if rule_enabled(config, VITEST_PROJECT_MAPPING) {
        scope.spawn(move |_| {
            let result = super::trace_rule(sources, VITEST_PROJECT_MAPPING, || {
                vitest_project_mapping::check_with_files_and_catalog(
                    root,
                    config,
                    candidates.candidates(VITEST_PROJECT_MAPPING),
                    vitest_catalog,
                )
            });
            acc.lock()
                .expect("mutex poisoned")
                .push((VITEST_PROJECT_MAPPING, result));
        });
    }
    if rule_enabled(config, VITEST_CI_PATH_COVERAGE) {
        scope.spawn(move |_| { let result = super::trace_rule(sources, VITEST_CI_PATH_COVERAGE, || vitest_ci_path_coverage::check_with_files_from_snapshot_catalog_sources_and_workflows(root, config, candidates.candidates(VITEST_CI_PATH_COVERAGE), snapshot, vitest_catalog, sources, workflow_documents)); acc.lock().expect("mutex poisoned").push((VITEST_CI_PATH_COVERAGE, result)); });
    }
    if rule_enabled(config, TSCONFIG_GATE_COVERAGE) {
        scope.spawn(move |_| {
            let result = super::trace_rule(sources, TSCONFIG_GATE_COVERAGE, || {
                workflow_documents
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
                        },
                    )
            });
            acc.lock()
                .expect("mutex poisoned")
                .push((TSCONFIG_GATE_COVERAGE, result));
        });
    }
}
