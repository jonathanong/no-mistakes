struct FinalizeInputs<'a> {
    framework: TestFramework,
    root: &'a Path,
    config: &'a NoMistakesConfig,
    changed_files: &'a [PathBuf],
    deleted_files: &'a [PathBuf],
    discovered_tests: &'a DiscoveredTests,
    native_traceable_changed_files: HashSet<String>,
    native_semantic_seeds: native_semantic_seeds::NativeSemanticSeedResult,
    root_visible_paths: &'a [PathBuf],
    global_config_fallback: bool,
    global_limit: usize,
    has_global_limit: bool,
    lockfile_seed_result: Option<lockfile_seeds::LockfileSeedResult>,
    vitest_setup_fallback: VitestSetupFallback<'a>,
    selection: group_selection_types::GroupSelectionResult,
}

fn finish_configured_plan(inputs: FinalizeInputs<'_>) -> Result<TestPlan> {
    let FinalizeInputs {
        framework,
        root,
        config,
        changed_files,
        deleted_files,
        discovered_tests,
        native_traceable_changed_files,
        native_semantic_seeds,
        root_visible_paths,
        global_config_fallback,
        global_limit,
        has_global_limit,
        lockfile_seed_result,
        vitest_setup_fallback,
        selection,
    } = inputs;
    let all_tests = &discovered_tests.tests;
    let mut selected_map = selection.selected_map;
    let mut used = selection.used;
    let mut group_results = selection.group_results;
    let remaining_global = selection.remaining_global;
    let mut warnings = selection.warnings;
    let mut warnings_seen = selection.warnings_seen;
    let mut fallback_reasons = selection.fallback_reasons;
    let lockfile_seeds_injected = selection.lockfile_seeds_injected;
    native_semantic_seeds.extend_warnings(&mut warnings, &mut warnings_seen);
    if let Some(plan) = native_semantic_fallback_plan(
        root,
        all_tests,
        &native_semantic_seeds,
        global_config_fallback,
        global_limit,
        has_global_limit,
        &warnings,
        discovered_tests,
    ) {
        return Ok(plan);
    }

    if let Some(seed_result) = lockfile_seed_result {
        for dependency in &seed_result.untraceable_dependencies {
            let warning = Warning {
                r#type: "package-dependency-untraceable".to_string(),
                message: format!(
                    "`{}` changed `{}` without a causal path to a configured test; full-suite selection requires global fallback opt-in",
                    dependency.lockfile, dependency.package_name,
                ),
                file: dependency.lockfile.clone(),
                line: None,
            };
            if warnings_seen.insert(warning_key(&warning)) {
                warnings.push(warning);
            }
        }
        if lockfile_seeds_injected {
            if !seed_result.untraceable_dependencies.is_empty() && global_config_fallback {
                let dependency = &seed_result.untraceable_dependencies[0];
                let changed_file = root.join(&dependency.lockfile);
                let msg = format!(
                    "`{}` changed transitive dependency `{}`; falling back to full test suite",
                    dependency.lockfile, dependency.package_name,
                );
                let mut plan = fallback_plan(
                    root,
                    all_tests,
                    FallbackRequest {
                        group_type: "dependencies",
                        via: "transitive dependency",
                        changed_file: Some(&changed_file),
                        limit: global_limit,
                        has_limit: has_global_limit,
                        reason: msg,
                    },
                );
                plan.warnings.extend(warnings);
                attach_targets(&mut plan, root, discovered_tests);
                return Ok(plan);
            }
        } else {
            if let Some(mut fallback) = apply_lockfile_seeds(
                root,
                seed_result,
                global_config_fallback,
                all_tests,
                global_limit,
                has_global_limit,
                &mut selected_map,
                &mut used,
                &mut group_results,
                discovered_tests,
            )? {
                fallback.warnings.extend(warnings);
                return Ok(fallback);
            }
        }
    }

    if !all_tests.is_empty() {
        if let Some((reason, picked)) = native_fallback_selection(
            framework,
            root,
            config,
            changed_files,
            deleted_files,
            &selected_map,
            &native_traceable_changed_files,
            &used,
            all_tests,
            discovered_tests,
            root_visible_paths,
            global_config_fallback,
            remaining_global,
        ) {
            for test in &picked {
                used.insert(test.test_file.clone());
                selected_map
                    .entry(root.join(&test.test_file))
                    .and_modify(|entry| merge_selected(entry, test))
                    .or_insert_with(|| test.clone());
            }
            if !picked.is_empty() {
                group_results.push(TestPlanGroupResult {
                    r#type: "dependencies".to_string(),
                    selected: picked.iter().map(|test| test.test_file.clone()).collect(),
                    remaining: all_tests.len().saturating_sub(used.len()),
                    limit: has_global_limit.then_some(remaining_global),
                });
            }
            fallback_reasons.push(reason);
        }
    }

    if !vitest_setup_fallback.checked_dependency_group() {
        let remaining_global = global_limit.saturating_sub(used.len());
        vitest_setup_fallback.apply_without_dependency_group(
            VitestSetupSelection {
                used: &mut used,
                selected_map: &mut selected_map,
                group_results: &mut group_results,
                fallback_reasons: &mut fallback_reasons,
            },
            remaining_global,
        );
    }

    let mut plan = TestPlan {
        changed_files: Vec::new(),
        selected_tests: sorted_selected_tests(selected_map),
        groups: group_results,
        warnings: sorted_warnings(warnings),
        fallback_triggered: !fallback_reasons.is_empty(),
        fallback_reason: (!fallback_reasons.is_empty()).then(|| fallback_reasons.join("; ")),
        ..Default::default()
    };
    attach_targets(&mut plan, root, discovered_tests);
    Ok(plan)
}
