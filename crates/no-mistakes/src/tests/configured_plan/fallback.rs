use crate::tests::configured_plan_candidates::{first_take, selected_from_paths};
use crate::tests::plan::relative_path;
use crate::tests::{SelectedTest, TestPlan, TestPlanGroupResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct FallbackRequest<'a> {
    pub(super) group_type: &'a str,
    pub(super) via: &'a str,
    pub(super) changed_file: Option<&'a PathBuf>,
    pub(super) limit: usize,
    pub(super) has_limit: bool,
    pub(super) reason: String,
}

pub(super) fn fallback_plan(
    root: &Path,
    all_tests: &[PathBuf],
    request: FallbackRequest<'_>,
) -> TestPlan {
    let effective_limit = request.limit.min(all_tests.len());
    let selected_tests = first_take(
        selected_from_paths(root, all_tests, request.via, request.changed_file),
        effective_limit,
    );
    let group = fallback_group(
        root,
        request.group_type,
        all_tests,
        &selected_tests,
        request.has_limit.then_some(effective_limit),
    );
    TestPlan {
        selected_tests,
        groups: vec![group],
        warnings: Vec::new(),
        fallback_triggered: true,
        fallback_reason: Some(request.reason),
    }
}

fn fallback_group(
    root: &Path,
    group_type: &str,
    all_tests: &[PathBuf],
    selected_tests: &[SelectedTest],
    limit: Option<usize>,
) -> TestPlanGroupResult {
    let selected: Vec<String> = selected_tests
        .iter()
        .map(|test| test.test_file.clone())
        .collect();
    let selected_set: HashSet<&str> = selected.iter().map(String::as_str).collect();
    let remaining = all_tests
        .iter()
        .filter(|test| {
            let rel = relative_path(root, test);
            !selected_set.contains(rel.as_str())
        })
        .count();
    TestPlanGroupResult {
        r#type: group_type.to_string(),
        selected,
        remaining,
        limit,
    }
}
