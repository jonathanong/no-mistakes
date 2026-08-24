use super::*;

/// Merge lockfile-seeded candidates into `selected_map` and `group_results`, or return
/// a full-suite fallback plan when `global_config_fallback` is set and there are
/// genuinely untraceable tooling deps.
#[allow(clippy::too_many_arguments)]
pub(in crate::tests::configured_plan) fn apply_lockfile_seeds(
    root: &Path,
    seed_result: LockfileSeedResult,
    global_config_fallback: bool,
    all_tests: &[PathBuf],
    global_limit: usize,
    has_global_limit: bool,
    selected_map: &mut BTreeMap<PathBuf, SelectedTest>,
    used: &mut HashSet<String>,
    group_results: &mut Vec<TestPlanGroupResult>,
    discovered_tests: &DiscoveredTests,
) -> Result<Option<TestPlan>> {
    if !seed_result.untraceable_lockfiles.is_empty() && global_config_fallback {
        let lf = &seed_result.untraceable_lockfiles[0];
        let changed_file = root.join(lf);
        let msg = format!(
            "`{}` changed a transitive dependency; falling back to full test suite",
            lf
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
        super::super::attach_targets(&mut plan, root, discovered_tests);
        return Ok(Some(plan));
    }
    let mut max_to_add = if has_global_limit {
        global_limit.saturating_sub(used.len())
    } else {
        usize::MAX
    };
    if let Some(dep_group) = group_results.iter().find(|g| g.r#type == "dependencies") {
        if let Some(limit_val) = dep_group.limit {
            max_to_add = max_to_add.min(limit_val.saturating_sub(dep_group.selected.len()));
        }
    }
    let mut added = Vec::new();
    for test in &seed_result.candidates {
        if used.contains(&test.test_file) {
            if let Some(existing) = selected_map.get_mut(&root.join(&test.test_file)) {
                merge_selected(existing, test);
                existing.targets.clear();
            }
            continue;
        }
        if added.len() >= max_to_add {
            continue;
        }
        if used.insert(test.test_file.clone()) {
            selected_map
                .entry(root.join(&test.test_file))
                .and_modify(|entry| merge_selected(entry, test))
                .or_insert_with(|| test.clone());
            added.push(test.test_file.clone());
        }
    }
    if !added.is_empty() {
        if let Some(dep_group) = group_results
            .iter_mut()
            .find(|g| g.r#type == "dependencies")
        {
            for name in added {
                if !dep_group.selected.contains(&name) {
                    dep_group.selected.push(name);
                }
            }
            dep_group.remaining = all_tests.len().saturating_sub(used.len());
        } else {
            group_results.push(TestPlanGroupResult {
                r#type: "dependencies".to_string(),
                selected: added,
                remaining: all_tests.len().saturating_sub(used.len()),
                limit: None,
            });
        }
    }
    Ok(None)
}
