use super::configured_plan_candidates::{group_candidates, merge_selected};
use super::diff_parser::DiffFile;
use super::plan::relative_path;
use super::{
    push_resource_diagnostics, warning_key, PlanArgs, SelectedTest, TestFramework, TestPlan,
    TestPlanGroupResult, Warning, WarningKey,
};
use anyhow::Result;
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::codebase::workspaces::WorkspaceMap;
use no_mistakes::config::v2::schema::{NoMistakesConfig, TestPlanGroupType};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

mod dep_triggers;
mod discovery;
mod environment;
mod fallback;
mod finalize;
mod hints;
mod hints_domains;
mod lockfile_seeds;
mod native_fallback;
mod targeted_triggers;
#[cfg(test)]
mod tests;
pub(super) mod vitest_setup_fallback;
mod vitest_setup_groups;
use dep_triggers::dependency_triggers;
pub(crate) use discovery::discover_framework_tests_from_prepared;
use environment::{
    configured_environment, configured_groups, effective_global_config_fallback, framework_name,
    group_type_name, limit_count, override_limit,
};
use fallback::{fallback_plan, FallbackRequest};
use finalize::{
    attach_targets, empty_group_result, select_limited_group_candidates, sorted_selected_tests,
    sorted_warnings,
};
use hints::build_coverage_hints_from_prepared;
use lockfile_seeds::{
    apply_lockfile_seeds, lockfile_seed_candidates, merge_lockfile_seed_candidates,
};
use native_fallback::{native_fallback_selection, native_traceable_changed_files};
use targeted_triggers::{
    insert_synthesized_dependency_group, merge_targeted_candidates, targeted_dependency_candidates,
    TargetedOverlapRecovery,
};
use vitest_setup_groups::{VitestSetupFallback, VitestSetupFallbackInputs, VitestSetupSelection};

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_configured_plan_with_prepared(
    args: &PlanArgs,
    framework: TestFramework,
    root: &Path,
    config: &NoMistakesConfig,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
    diff_files: &[DiffFile],
    lockfile_changed_packages: &[(String, String)], // (pkg_name, lockfile_rel)
    workspace_map: &WorkspaceMap,
    forced_fallback: Option<(String, PathBuf)>,
    discovered_tests: DiscoveredTests,
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
    // Validate every structured target before any fallback or `all` environment
    // can return a plan successfully.
    let dependency_triggers =
        dependency_triggers(root, config, framework, changed_files, prepared)?;

    if effective_global_config_fallback(&env, args) {
        if let Some((reason, trigger_file)) = forced_fallback.as_ref() {
            let mut plan = fallback_plan(
                root,
                &all_tests,
                FallbackRequest {
                    group_type: "global",
                    via: "global configuration",
                    changed_file: Some(trigger_file),
                    limit: global_limit,
                    has_limit: has_global_limit,
                    reason: reason.clone(),
                },
            );
            plan.warnings.extend(vitest_setup_warnings);
            attach_targets(&mut plan, root, &discovered_tests);
            plan.warnings.extend(prepared.tsconfig_warnings());
            return Ok(plan);
        }
    }

    if env.all {
        let mut plan = fallback_plan(
            root,
            &all_tests,
            FallbackRequest {
                group_type: "all",
                via: "all",
                changed_file: changed_files.first(),
                limit: global_limit,
                has_limit: has_global_limit,
                reason: format!(
                    "{} test plan environment `{}` runs all tests",
                    framework_name(framework),
                    args.environment
                ),
            },
        );
        plan.warnings.extend(vitest_setup_warnings);
        attach_targets(&mut plan, root, &discovered_tests);
        plan.warnings.extend(prepared.tsconfig_warnings());
        return Ok(plan);
    }

    if let Some((reason, trigger_file)) = dependency_triggers.fallback {
        let mut plan = fallback_plan(
            root,
            &all_tests,
            FallbackRequest {
                group_type: "dependencies",
                via: "dependency configuration",
                changed_file: Some(&trigger_file),
                limit: global_limit,
                has_limit: has_global_limit,
                reason,
            },
        );
        plan.warnings.extend(vitest_setup_warnings);
        attach_targets(&mut plan, root, &discovered_tests);
        plan.warnings.extend(prepared.tsconfig_warnings());
        return Ok(plan);
    }

    let graph = if prepared.graph_is_initialized() {
        prepared.graph()?
    } else if let Some(timing) = timing {
        timing.run_phase("graph", || prepared.graph())?
    } else {
        prepared.graph()?
    };
    let test_filter = prepared.test_filter().clone();
    let coverage_hints =
        build_coverage_hints_from_prepared(prepared, config, framework, diff_files, &all_tests);
    let mut selected_map: BTreeMap<PathBuf, SelectedTest> = BTreeMap::new();
    let mut used = HashSet::new();
    let mut group_results = Vec::new();
    let mut remaining_global = global_limit;
    let mut warnings: Vec<Warning> = prepared.tsconfig_warnings();
    let mut warnings_seen: HashSet<WarningKey> = warnings.iter().map(warning_key).collect();
    for changed in changed_files {
        push_resource_diagnostics(graph, root, changed, &mut warnings, &mut warnings_seen);
    }
    warnings.extend(vitest_setup_warnings);
    let mut warnings_seen: HashSet<WarningKey> = warnings.iter().map(warning_key).collect();
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
    // §lockfile: pre-compute seeds before the group loop so they can be injected during
    // the dependencies group turn — before later groups (e.g. sample) consume the budget.
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
    let mut lockfile_seeds_injected = false;
    let targeted_candidates = targeted_dependency_candidates(
        root,
        &all_tests,
        &discovered_tests,
        &dependency_triggers.targeted,
    );
    let mut groups = configured_groups(&env, framework);
    let target_only_group_index =
        insert_synthesized_dependency_group(&mut groups, !targeted_candidates.is_empty());
    let mut targeted_overlaps = TargetedOverlapRecovery::new(&targeted_candidates);
    let mut fallback_reasons = Vec::new();
    let mut vitest_setup_fallback = VitestSetupFallback::new(VitestSetupFallbackInputs {
        framework,
        root,
        changed_files,
        deleted_files,
        projects: prepared.vitest_projects(),
        discovered: &discovered_tests,
        has_global_limit,
        all_test_count: all_tests.len(),
    });

    for (group_index, group) in groups.iter().enumerate() {
        let recover_targeted_overlaps = targeted_overlaps.should_recover(framework, group.type_);
        let merge_zero_budget_targeted =
            group.type_ == TestPlanGroupType::Dependencies && !targeted_candidates.is_empty();
        if remaining_global == 0 && !recover_targeted_overlaps && !merge_zero_budget_targeted {
            let result_index = group_results.len();
            group_results.push(empty_group_result(
                group_type_name(group.type_),
                all_tests.len().saturating_sub(used.len()),
                has_global_limit.then_some(0),
            ));
            if group.type_ == TestPlanGroupType::Dependencies {
                vitest_setup_fallback.apply_dependency_group(
                    VitestSetupSelection {
                        used: &mut used,
                        selected_map: &mut selected_map,
                        group_results: &mut group_results,
                        fallback_reasons: &mut fallback_reasons,
                    },
                    result_index,
                    0,
                    0,
                );
            }
            continue;
        }
        if matches!(
            framework,
            TestFramework::Dotnet | TestFramework::Vitest | TestFramework::Swift
        ) && group.type_ == TestPlanGroupType::Coverage
        {
            anyhow::bail!(
                "{} test plans do not support the coverage group",
                framework_name(framework)
            );
        }
        let target_only_group = target_only_group_index == Some(group_index);
        let candidate_used =
            targeted_overlaps.candidate_used_override(&used, recover_targeted_overlaps);
        let mut candidates = if target_only_group {
            Vec::new()
        } else {
            group_candidates(
                group.type_,
                root,
                changed_files,
                graph,
                &all_tests,
                &all_test_set,
                &test_filter,
                candidate_used.as_ref().unwrap_or(&used),
                &coverage_hints,
                &mut warnings,
                &mut warnings_seen,
            )
        };
        if recover_targeted_overlaps {
            targeted_overlaps.merge_existing(root, &mut candidates, &used, &mut selected_map);
        }
        // Inject lockfile-seeded candidates during the dependencies group turn so they
        // compete for budget before later groups (e.g. sample) can consume it.
        if group.type_ == TestPlanGroupType::Dependencies {
            merge_targeted_candidates(
                root,
                &mut candidates,
                &targeted_candidates,
                &used,
                &mut selected_map,
            );
            if let Some(ref seed_result) = lockfile_seed_result {
                lockfile_seeds_injected = true;
                merge_lockfile_seed_candidates(
                    root,
                    &seed_result.candidates,
                    &mut candidates,
                    &used,
                    &mut selected_map,
                );
            }
        }
        let group_limit = group
            .limit
            .as_ref()
            .and_then(|limit| limit_count(Some(limit), all_tests.len()))
            .unwrap_or(remaining_global)
            .min(remaining_global);
        let picked =
            select_limited_group_candidates(candidates, group_limit, group.sample_when_limited);
        for test in &picked {
            used.insert(test.test_file.clone());
            selected_map
                .entry(root.join(&test.test_file))
                .and_modify(|entry| merge_selected(entry, test))
                .or_insert_with(|| test.clone());
        }
        remaining_global = remaining_global.saturating_sub(picked.len());
        let result_index = group_results.len();
        group_results.push(TestPlanGroupResult {
            r#type: group_type_name(group.type_).to_string(),
            selected: picked.iter().map(|test| test.test_file.clone()).collect(),
            remaining: all_tests.len().saturating_sub(used.len()),
            limit: group
                .limit
                .is_some()
                .then_some(group_limit)
                .or_else(|| has_global_limit.then_some(group_limit)),
        });
        if group.type_ == TestPlanGroupType::Dependencies {
            let picked = vitest_setup_fallback.apply_dependency_group(
                VitestSetupSelection {
                    used: &mut used,
                    selected_map: &mut selected_map,
                    group_results: &mut group_results,
                    fallback_reasons: &mut fallback_reasons,
                },
                result_index,
                group_limit.saturating_sub(picked.len()),
                remaining_global,
            );
            remaining_global = remaining_global.saturating_sub(picked);
        }
        targeted_overlaps.finish_group(group.type_);
    }

    if let Some(seed_result) = lockfile_seed_result {
        if lockfile_seeds_injected {
            // Seeds were merged during the dependencies group turn.
            // Only handle the untraceable-dep fallback here.
            if !seed_result.untraceable_lockfiles.is_empty()
                && effective_global_config_fallback(&env, args)
            {
                let lf = &seed_result.untraceable_lockfiles[0];
                let msg = format!(
                    "`{}` changed a transitive dependency; falling back to full test suite",
                    lf
                );
                let mut plan = fallback_plan(
                    root,
                    &all_tests,
                    FallbackRequest {
                        group_type: "dependencies",
                        via: "transitive dependency",
                        changed_file: None,
                        limit: global_limit,
                        has_limit: has_global_limit,
                        reason: msg,
                    },
                );
                // This return occurs after normal warning initialization, so
                // retain diagnostics for unsafe Vitest setup declarations.
                plan.warnings.extend(warnings);
                attach_targets(&mut plan, root, &discovered_tests);
                return Ok(plan);
            }
        } else {
            // Custom config without a dependencies group: fall back to post-loop injection.
            if let Some(mut fallback) = apply_lockfile_seeds(
                root,
                seed_result,
                effective_global_config_fallback(&env, args),
                &all_tests,
                global_limit,
                has_global_limit,
                &mut selected_map,
                &mut used,
                &mut group_results,
                &discovered_tests,
            )? {
                // `apply_lockfile_seeds` owns its fallback plan; attach the
                // request-scoped setup diagnostics before its early return.
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
            &all_tests,
            &discovered_tests,
            prepared.root_visible_paths(),
            effective_global_config_fallback(&env, args),
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
        selected_tests: sorted_selected_tests(selected_map),
        groups: group_results,
        warnings: sorted_warnings(warnings),
        fallback_triggered: !fallback_reasons.is_empty(),
        fallback_reason: (!fallback_reasons.is_empty()).then(|| fallback_reasons.join("; ")),
    };
    attach_targets(&mut plan, root, &discovered_tests);
    Ok(plan)
}
