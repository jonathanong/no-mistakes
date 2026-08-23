use super::{GroupedExecutionTarget, TestPlan};
use no_mistakes::codebase::test_discovery::TestExecutionTarget;
use std::collections::BTreeMap;

impl TestPlan {
    pub(crate) fn finish(&mut self, include_comment: bool, prefixes: &[String]) {
        self.execution_targets = grouped_execution_targets(&self.selected_tests, prefixes);
        if include_comment {
            self.comment = Some(super::comment::render_markdown_plan(self));
        }
    }
}

type ExecutionGroupKey = (
    String,
    Option<String>,
    Option<String>,
    Vec<String>,
    Option<String>,
);

fn grouped_execution_targets(
    selected: &[super::SelectedTest],
    prefixes: &[String],
) -> Vec<GroupedExecutionTarget> {
    let mut groups: BTreeMap<ExecutionGroupKey, GroupedExecutionTarget> = BTreeMap::new();
    for test in selected {
        for target in &test.targets {
            let name = if target.runner == "swift" {
                prefix_name(&test.test_file, prefixes)
            } else {
                None
            };
            let key = (
                target.runner.clone(),
                target.config.clone(),
                target.project.clone(),
                target.base_command.clone(),
                name.clone(),
            );
            let group = groups.entry(key).or_insert_with(|| GroupedExecutionTarget {
                runner: target.runner.clone(),
                config: target.config.clone(),
                project: target.project.clone(),
                name,
                base_command: target.base_command.clone(),
                runner_args: runner_args_without_file(target, &test.test_file),
                test_files: Vec::new(),
            });
            if !group.test_files.iter().any(|file| file == &test.test_file) {
                group.test_files.push(test.test_file.clone());
            }
        }
    }
    groups.into_values().collect()
}

fn prefix_name(file: &str, prefixes: &[String]) -> Option<String> {
    prefixes
        .iter()
        .map(|prefix| prefix.trim_end_matches('/'))
        .filter(|prefix| !prefix.is_empty())
        .filter(|prefix| file == *prefix || file.starts_with(&format!("{prefix}/")))
        .max_by_key(|prefix| prefix.len())
        .map(str::to_string)
}

fn runner_args_without_file(target: &TestExecutionTarget, test_file: &str) -> Vec<String> {
    let mut args = target.runner_args.clone();
    if args.len() >= 2 && args[args.len() - 2] == "--test" {
        return args;
    }
    if args
        .last()
        .is_some_and(|last| last_arg_selects_test_file(last, test_file))
    {
        args.pop();
    }
    args
}

fn last_arg_selects_test_file(last: &str, test_file: &str) -> bool {
    let file = test_file.replace('\\', "/");
    let last = last.replace('\\', "/");
    if last == file || last == regex_escape(&file) {
        return true;
    }
    let as_module = file.strip_suffix(".py").unwrap_or(&file).replace('/', ".");
    if last == as_module {
        return true;
    }
    if file.ends_with(&format!("/{last}")) {
        return true;
    }
    if let Some(dir) = file.rsplit_once('/').map(|(dir, _)| dir) {
        if last == format!("./{dir}") || last.trim_start_matches("./") == dir {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "plan_finish/tests.rs"]
mod tests;

fn regex_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}
