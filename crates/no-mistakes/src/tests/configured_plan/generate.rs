#[expect(
    clippy::too_many_arguments,
    reason = "The plan entrypoint borrows prepared analysis alongside diff, workspace, discovery, and timing inputs."
)]
pub(crate) fn generate_configured_plan_with_prepared(
    args: &PlanArgs,
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
    diff_files: &[DiffFile],
    lockfile_changed_packages: &[(String, String, Vec<PathBuf>)],
    workspace_map: &WorkspaceMap,
    forced_fallback: Option<(String, PathBuf)>,
    discovered_tests: &DiscoveredTests,
    prepared: &super::prepared_plan::PreparedTestPlanRequest,
    timing: Option<&mut crate::impacted_checks::timing::TimingTracker>,
) -> Result<TestPlan> {
    let env = configured_environment(args, framework, config)?;
    let all_tests = discovered_tests.tests.clone();
    let vitest_setup_warnings =
        vitest_setup_fallback::framework_warnings(framework, root, prepared.vitest_projects());
    let all_test_set: HashSet<PathBuf> = all_tests.iter().cloned().collect();
    let effective_limit = override_limit(env.limit.as_ref(), args);
    let has_global_limit = effective_limit.is_some();
    let global_limit =
        limit_count(effective_limit.as_ref(), all_tests.len()).unwrap_or(all_tests.len());
    let dependency_triggers = dependency_triggers(
        root,
        config,
        framework,
        changed_files,
        &all_test_set,
        prepared,
    )?;

    let global_config_fallback = effective_global_config_fallback(&env, args);
    if let Some(request) = early_fallback_request(
        args,
        framework,
        &env,
        global_config_fallback,
        changed_files,
        forced_fallback.as_ref(),
        dependency_triggers.fallback.as_ref(),
        (global_limit, has_global_limit),
    ) {
        let mut plan = fallback_plan(root, &all_tests, request);
        plan.warnings.extend(vitest_setup_warnings);
        attach_targets(&mut plan, root, discovered_tests);
        plan.warnings.extend(prepared.tsconfig_warnings());
        return Ok(plan);
    }

    let graph = graph::prepared_graph(prepared, timing)?;
    let test_filter = prepared.test_filter().clone();
    let coverage_hints =
        build_coverage_hints_from_prepared(prepared, config, framework, diff_files, &all_tests);
    let mut warnings: Vec<Warning> = prepared.tsconfig_warnings();
    let mut warnings_seen: HashSet<WarningKey> = warnings.iter().map(warning_key).collect();
    for changed in changed_files {
        push_resource_diagnostics(graph, root, changed, &mut warnings, &mut warnings_seen);
    }
    warnings.extend(vitest_setup_warnings);
    let warnings_seen: HashSet<WarningKey> = warnings.iter().map(warning_key).collect();
    let native_traceable_changed_files = native_traceable_changed_files(
        framework,
        root,
        changed_files,
        graph,
        &all_tests,
        &all_test_set,
        &test_filter,
        &coverage_hints,
    );
    let lockfile_seed_result = if lockfile_changed_packages.is_empty() {
        None
    } else {
        Some(lockfile_seed_candidates(
            root,
            lockfile_changed_packages,
            workspace_map,
            graph,
            &all_test_set,
            &HashSet::new(), // filter against `used` during injection, not pre-compute
        ))
    };
    let native_semantic_seeds =
        native_semantic_seed_candidates(root, prepared, graph, &all_test_set, Some(framework));
    let targeted_candidates = targeted_dependency_candidates(
        root,
        &all_tests,
        discovered_tests,
        &dependency_triggers.targeted,
    );
    let mut groups = configured_groups(&env, framework);
    let has_semantic_dependency_candidates = !targeted_candidates.is_empty()
        || lockfile_seed_result
            .as_ref()
            .is_some_and(|result| !result.candidates.is_empty())
        || !native_semantic_seeds.candidates.is_empty();
    let target_only_group_index =
        insert_synthesized_dependency_group(&mut groups, has_semantic_dependency_candidates);
    let mut vitest_setup_fallback = VitestSetupFallback::new(VitestSetupFallbackInputs {
        framework,
        root,
        changed_files,
        deleted_files,
        projects: prepared.vitest_projects(),
        discovered: discovered_tests,
        has_global_limit,
        all_test_count: all_tests.len(),
    });
    let selection = select_configured_groups(GroupSelectionInputs {
        framework,
        root,
        changed_files,
        graph,
        all_tests: &all_tests,
        all_test_set: &all_test_set,
        test_filter: &test_filter,
        coverage_hints: &coverage_hints,
        groups: &groups,
        target_only_group_index,
        targeted_candidates: &targeted_candidates,
        lockfile_seed_result: lockfile_seed_result.as_ref(),
        native_semantic_candidates: &native_semantic_seeds.candidates,
        has_global_limit,
        remaining_global: global_limit,
        warnings,
        warnings_seen,
        vitest_setup_fallback: &mut vitest_setup_fallback,
    })?;
    finish_configured_plan(FinalizeInputs {
        framework,
        root,
        config,
        changed_files,
        deleted_files,
        discovered_tests,
        native_traceable_changed_files,
        native_semantic_seeds,
        root_visible_paths: prepared.root_visible_paths(),
        global_config_fallback,
        global_limit,
        has_global_limit,
        lockfile_seed_result,
        vitest_setup_fallback,
        selection,
    })
}
