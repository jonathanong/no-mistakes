use super::publish::publish_hits;
use super::{gha_kind, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use crate::codebase::rules::github_actions_workflow_files::run_scripts::run_scripts;
use serde_yaml::Value;

struct Body {
    job: String,
    step: usize,
    script: String,
    composite: bool,
    target: String,
}

pub(super) fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    action: bool,
) -> Vec<RuleFinding> {
    let located = step_run_lines(source, action);
    let mut findings = Vec::new();
    if !action {
        if let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) {
            findings.extend(super::ports::service_findings(rel, source, jobs));
        }
    }
    for body in bodies(action, value) {
        let lines = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        findings.extend(judge(rel, &body, lines));
    }
    findings
}

fn bodies(action: bool, value: &Value) -> Vec<Body> {
    run_scripts(action, value)
        .into_iter()
        .map(|script| Body {
            target: if script.composite {
                "composite".to_string()
            } else {
                script.job.clone()
            },
            job: script.job,
            step: script.step,
            script: script.script,
            composite: script.composite,
        })
        .collect()
}

fn judge(rel: &str, body: &Body, located: Option<&StepRun>) -> Vec<RuleFinding> {
    let place = if body.composite {
        format!("composite step {}", body.step)
    } else {
        format!("job `{}` step {}", body.target, body.step)
    };
    publish_hits(&body.script)
        .into_iter()
        .map(|hit| {
            let line = source_line(located, &body.script, hit.line, &hit.spec);
            RuleFinding {
                causes: Vec::new(),
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line,
                message: pin_message(rel, line, &place, &hit.host, &hit.spec),
                import: None,
                target: Some(body.target.clone()),
            }
        })
        .collect()
}

pub(super) fn pin_message(rel: &str, line: usize, place: &str, host: &str, spec: &str) -> String {
    format!(
        "{rel}:{line}: {place} pins host port `{host}` in `{spec}`. \
         Hosted and shared runners collide when a publish mapping \
         fixes the host port. Publish a Docker-allocated port \
         (`127.0.0.1::5432`, `::5432`, `0:5432`, or the container port only). \
         A variable host port is not judged."
    )
}

fn source_line(located: Option<&StepRun>, script: &str, script_line: usize, spec: &str) -> usize {
    let Some(located) = located.filter(|item| !item.lines.is_empty()) else {
        return 1;
    };
    if script.lines().count() == located.lines.len() {
        let index = script_line.saturating_sub(1);
        return located.lines.get(index).map(|(line, _)| *line).unwrap_or(1);
    }
    located
        .lines
        .iter()
        .find(|(_, text)| text.contains(spec))
        .map(|(line, _)| *line)
        .unwrap_or(located.lines[0].0)
}
