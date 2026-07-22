use crate::tests::configured_plan_candidates::{first_take, stable_take};
use crate::tests::{SelectedTest, TestPlanGroupResult, Warning};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn sorted_selected_tests(
    selected_map: BTreeMap<PathBuf, SelectedTest>,
) -> Vec<SelectedTest> {
    let mut selected_tests: Vec<SelectedTest> = selected_map.into_values().collect();
    for test in &mut selected_tests {
        test.reasons
            .sort_by(|a, b| a.changed_file.cmp(&b.changed_file));
    }
    selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
    selected_tests
}

pub(super) fn sorted_warnings(mut warnings: Vec<Warning>) -> Vec<Warning> {
    warnings.sort_by(|a, b| {
        (&a.file, a.line, &a.r#type, &a.message).cmp(&(&b.file, b.line, &b.r#type, &b.message))
    });
    warnings
}

pub(super) fn empty_group_result(
    group_type: &str,
    remaining: usize,
    limit: Option<usize>,
) -> TestPlanGroupResult {
    TestPlanGroupResult {
        r#type: group_type.to_string(),
        selected: Vec::new(),
        remaining,
        limit,
    }
}

pub(super) fn select_limited_group_candidates(
    candidates: Vec<SelectedTest>,
    limit: usize,
    sample_when_limited: bool,
) -> Vec<SelectedTest> {
    if limit == 0 {
        Vec::new()
    } else if sample_when_limited && candidates.len() > limit {
        stable_take(candidates, limit)
    } else {
        first_take(candidates, limit)
    }
}
