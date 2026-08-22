use super::{GroupedExecutionTarget, TestPlan};
use no_mistakes::codebase::test_discovery::TestExecutionTarget;
use std::collections::BTreeMap;

impl TestPlan {
    pub(crate) fn finish(&mut self, include_comment: bool) {
        self.execution_targets = grouped_execution_targets(&self.selected_tests);
        if include_comment {
            self.comment = Some(super::comment::render_markdown_plan(self));
        }
    }
}

fn grouped_execution_targets(selected: &[super::SelectedTest]) -> Vec<GroupedExecutionTarget> {
    let mut groups: BTreeMap<(String, Option<String>, Option<String>), GroupedExecutionTarget> =
        BTreeMap::new();
    for test in selected {
        let Some(target) = test.targets.first() else {
            continue;
        };
        let key = (
            target.runner.clone(),
            target.config.clone(),
            target.project.clone(),
        );
        let group = groups.entry(key).or_insert_with(|| GroupedExecutionTarget {
            runner: target.runner.clone(),
            config: target.config.clone(),
            project: target.project.clone(),
            base_command: target.base_command.clone(),
            runner_args: runner_args_without_file(target, &test.test_file),
            test_files: Vec::new(),
        });
        if !group.test_files.iter().any(|file| file == &test.test_file) {
            group.test_files.push(test.test_file.clone());
        }
    }
    groups.into_values().collect()
}

fn runner_args_without_file(target: &TestExecutionTarget, test_file: &str) -> Vec<String> {
    let mut args = target.runner_args.clone();
    if args.last().is_some_and(|last| last == test_file) {
        args.pop();
    }
    args
}
