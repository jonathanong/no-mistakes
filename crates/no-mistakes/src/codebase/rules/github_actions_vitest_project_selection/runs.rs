use super::judge::{narrowings, Narrowing};
use super::options::{Compiled, NameFilters};
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::codebase::rules::github_actions_workflow_files::run_scripts::{run_scripts, RunScript};
use no_mistakes_rule_kernel::finding::RuleFinding;
use serde_yaml::Value;
use std::collections::BTreeMap;

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    action: bool,
    compiled: &Compiled,
) -> Vec<RuleFinding> {
    let policies = policies_for(rel, compiled);
    let located = step_run_lines(source, action);
    run_scripts(action, value)
        .into_iter()
        .flat_map(|body| judge(rel, source, &body, &policies, &located))
        .collect()
}

fn policies_for<'a>(rel: &str, compiled: &'a Compiled) -> BTreeMap<&'a str, NameFilters> {
    compiled
        .projects
        .iter()
        .filter(|policy| policy.workflows.is_match(rel))
        .map(|policy| (policy.project.as_str(), policy.name_filters))
        .collect()
}

fn judge(
    rel: &str,
    source: &str,
    body: &RunScript,
    policies: &BTreeMap<&str, NameFilters>,
    located: &[StepRun],
) -> Vec<RuleFinding> {
    let place = if body.composite {
        format!("composite step {}", body.step)
    } else {
        format!("job `{}` step {}", body.job, body.step)
    };
    let lines = located
        .iter()
        .find(|item| item.job == body.job && item.step == body.step);
    narrowings(&body.script, policies)
        .into_iter()
        .map(|hit| finding(rel, source, &place, &body.script, lines, hit))
        .collect()
}

fn finding(
    rel: &str,
    source: &str,
    place: &str,
    script: &str,
    located: Option<&StepRun>,
    hit: Narrowing,
) -> RuleFinding {
    let line = report_line(source, located, script, &hit);
    let detail = if hit.flag {
        format!("test name filter `{}`", hit.argument)
    } else if hit.name {
        format!("positional test name `{}`", hit.argument)
    } else {
        format!("positional test file `{}`", hit.argument)
    };
    let fix = if hit.name {
        "Drop the name filter, or set nameFilters to allow when a non-file filter is intentional."
    } else {
        "Run the configured project without a positional file argument."
    };
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: format!(
            "{rel}:{line}: {place} narrows Vitest project `{}` with {detail}. \
             Whole-project selection is required for this project. {fix}",
            hit.project,
        ),
        import: None,
        target: Some(hit.argument),
    }
}

fn report_line(source: &str, located: Option<&StepRun>, script: &str, hit: &Narrowing) -> usize {
    let placed = yaml_line(located, script, hit.line, &hit.argument);
    if placed != 1 {
        return placed;
    }
    source
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(&hit.argument) && line.contains("run"))
        .map(|(index, _)| index + 1)
        .unwrap_or(placed)
}

pub(super) fn yaml_line(
    located: Option<&StepRun>,
    script: &str,
    script_line: usize,
    token: &str,
) -> usize {
    let Some(located) = located.filter(|item| !item.lines.is_empty()) else {
        return 1;
    };
    if script.lines().count() == located.lines.len() {
        return located
            .lines
            .get(script_line.saturating_sub(1))
            .map(|(line, _)| *line)
            .unwrap_or(located.lines[0].0);
    }
    located
        .lines
        .iter()
        .find(|(_, text)| text.contains(token))
        .map(|(line, _)| *line)
        .unwrap_or(located.lines[0].0)
}
