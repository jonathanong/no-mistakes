use super::super::configured_plan_candidates::{group_candidates, merge_selected};
use super::super::TestPlanGroupResult;
use super::coverage_support::reject_unsupported_coverage_group;
use super::group_selection_types::{GroupSelectionInputs, GroupSelectionResult};
use super::lockfile_seeds::merge_lockfile_seed_candidates;
use super::targeted_triggers::{merge_targeted_candidates, TargetedOverlapRecovery};
use super::vitest_setup_groups::VitestSetupSelection;
use no_mistakes::config::v2::schema::TestPlanGroupType;

pub(super) fn select_configured_groups(
    inputs: GroupSelectionInputs<'_, '_>,
) -> anyhow::Result<GroupSelectionResult> {
    let GroupSelectionInputs {
        framework,
        root,
        changed_files,
        graph,
        all_tests,
        all_test_set,
        test_filter,
        coverage_hints,
        groups,
        target_only_group_index,
        targeted_candidates,
        lockfile_seed_result,
        native_semantic_candidates,
        has_global_limit,
        mut remaining_global,
        mut warnings,
        mut warnings_seen,
        vitest_setup_fallback,
    } = inputs;
    let mut selected_map = std::collections::BTreeMap::new();
    let mut used = std::collections::HashSet::new();
    let mut group_results = Vec::new();
    let mut fallback_reasons = Vec::new();
    let mut lockfile_seeds_injected = false;
    let mut targeted_overlaps = TargetedOverlapRecovery::new(targeted_candidates);

    for (group_index, group) in groups.iter().enumerate() {
        let recover_targeted_overlaps = targeted_overlaps.should_recover(framework, group.type_);
        let merge_zero_budget_targeted =
            group.type_ == TestPlanGroupType::Dependencies && !targeted_candidates.is_empty();
        if remaining_global == 0 && !recover_targeted_overlaps && !merge_zero_budget_targeted {
            let result_index = group_results.len();
            group_results.push(super::finalize::empty_group_result(
                super::environment::group_type_name(group.type_),
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
        reject_unsupported_coverage_group(framework, group.type_)?;
        let target_only_group = target_only_group_index == Some(group_index);
        let candidate_used =
            targeted_overlaps.candidate_used_override(&used, recover_targeted_overlaps);
        let mut candidates = if target_only_group {
            Vec::new()
        } else {
            group_candidates(
                group.type_,
                framework,
                root,
                changed_files,
                graph,
                all_tests,
                all_test_set,
                test_filter,
                candidate_used.as_ref().unwrap_or(&used),
                coverage_hints,
                &mut warnings,
                &mut warnings_seen,
            )
        };
        if recover_targeted_overlaps {
            targeted_overlaps.merge_existing(root, &mut candidates, &used, &mut selected_map);
        }
        if group.type_ == TestPlanGroupType::Dependencies {
            merge_targeted_candidates(
                root,
                &mut candidates,
                targeted_candidates,
                &used,
                &mut selected_map,
            );
            if let Some(seed_result) = lockfile_seed_result {
                lockfile_seeds_injected = true;
                merge_lockfile_seed_candidates(
                    root,
                    &seed_result.candidates,
                    &mut candidates,
                    &used,
                    &mut selected_map,
                );
            }
            merge_lockfile_seed_candidates(
                root,
                native_semantic_candidates,
                &mut candidates,
                &used,
                &mut selected_map,
            );
        }
        let group_limit = group
            .limit
            .as_ref()
            .and_then(|limit| super::environment::limit_count(Some(limit), all_tests.len()))
            .unwrap_or(remaining_global)
            .min(remaining_global);
        let picked = if group.type_ == TestPlanGroupType::Direct {
            super::finalize::select_limited_direct_candidates(
                candidates,
                group_limit,
                group.sample_when_limited,
            )
        } else {
            super::finalize::select_limited_group_candidates(
                candidates,
                group_limit,
                group.sample_when_limited,
            )
        };
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
            r#type: super::environment::group_type_name(group.type_).to_string(),
            selected: picked.iter().map(|test| test.test_file.clone()).collect(),
            remaining: all_tests.len().saturating_sub(used.len()),
            limit: group
                .limit
                .is_some()
                .then_some(group_limit)
                .or_else(|| has_global_limit.then_some(group_limit)),
        });
        if group.type_ == TestPlanGroupType::Dependencies {
            let picked_count = vitest_setup_fallback.apply_dependency_group(
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
            remaining_global = remaining_global.saturating_sub(picked_count);
        }
        targeted_overlaps.finish_group(group.type_);
    }
    Ok(GroupSelectionResult {
        selected_map,
        used,
        group_results,
        remaining_global,
        warnings,
        warnings_seen,
        fallback_reasons,
        lockfile_seeds_injected,
    })
}
