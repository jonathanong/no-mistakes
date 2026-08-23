use super::*;
use crate::tests::{configured_plan, configured_plan_candidates, prepared_plan, TestFramework};

/// Apply Vitest's conservative setup handling after ordinary union tracing so
/// exact graph candidates retain their established priority.
pub(super) fn apply_union_fallback(
    prepared: &prepared_plan::PreparedTestPlanRequest,
    root: &Path,
    changed_files: &[PathBuf],
    deleted_files: &[PathBuf],
    selected_map: &mut HashMap<PathBuf, SelectedTest>,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<WarningKey>,
) -> Result<Option<String>> {
    if prepared.vitest_projects().is_none() {
        return Ok(None);
    }
    for warning in
        configured_plan::vitest_setup_fallback::warnings(root, prepared.vitest_projects())
    {
        if warnings_seen.insert(warning_key(&warning)) {
            warnings.push(warning);
        }
    }
    let discovered = prepared.discover_tests(TestFramework::Vitest)?;
    let used = selected_map
        .values()
        .map(|test| test.test_file.clone())
        .collect::<HashSet<_>>();
    let Some((reason, picked)) = configured_plan::vitest_setup_fallback::selection(
        root,
        changed_files,
        deleted_files,
        prepared.vitest_projects(),
        &discovered,
        &used,
        usize::MAX,
    ) else {
        return Ok(None);
    };
    for test in picked {
        selected_map
            .entry(root.join(&test.test_file))
            .and_modify(|existing| {
                configured_plan_candidates::merge_selected(existing, &test);
            })
            .or_insert(test);
    }
    Ok(Some(reason))
}
