use super::loops::poll_loops;
use super::RULE_ID;
use crate::codebase::rules::github_actions_script_expressions::step_run_lines;
use crate::codebase::rules::github_actions_workflow_files::gha_kind;
use crate::codebase::rules::github_actions_workflow_files::run_scripts::run_scripts;
use crate::codebase::rules::github_actions_workflow_files::script_line::yaml_line;
use crate::codebase::rules::RuleFinding;
use serde_yaml::Value;

pub(super) fn check_source(
    rel: &str,
    source: &str,
    polls: &[Vec<String>],
    skip_for: bool,
) -> Vec<RuleFinding> {
    let Some(action) = gha_kind(rel) else {
        return Vec::new();
    };
    let source = source.trim_start_matches('\u{feff}');
    let Ok(value) = serde_yaml::from_str::<Value>(source) else {
        return Vec::new();
    };
    let located = step_run_lines(source, action);
    let mut findings = Vec::new();
    for body in run_scripts(action, &value) {
        let lines = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        let place = if body.composite {
            format!("composite step {}", body.step)
        } else {
            format!("job `{}` step {}", body.job, body.step)
        };
        for hit in poll_loops(&body.script, polls, skip_for) {
            let line = yaml_line(lines, &body.script, hit.line, &hit.keyword);
            findings.push(RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line,
                message: message(rel, line, &place, &hit.keyword, &hit.poll),
                import: None,
                target: (!body.composite).then(|| body.job.clone()),
            });
        }
    }
    findings
}

fn message(rel: &str, line: usize, place: &str, keyword: &str, poll: &str) -> String {
    format!(
        "{rel}:{line}: {place} waits in a `{keyword}` loop that sleeps and runs `{poll}`. \
         A job that polls another job or workflow burns runner minutes. \
         Depend on the work with `needs`, or react to an event \
         (`workflow_run`, `merge_group`) instead of polling."
    )
}
