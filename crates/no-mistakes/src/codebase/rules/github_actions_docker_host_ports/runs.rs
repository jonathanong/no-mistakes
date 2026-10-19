use super::publish::publish_hits;
use super::{gha_kind, RuleFinding, RULE_ID};
use crate::codebase::rules::github_actions_script_expressions::{step_run_lines, StepRun};
use serde_yaml::{Mapping, Value};

struct Body {
    job: String,
    step: usize,
    script: String,
    composite: bool,
    target: String,
}

pub(super) fn check_source(rel: &str, source: &str) -> Vec<RuleFinding> {
    let Some(action) = gha_kind(rel) else {
        return Vec::new();
    };
    let source = source.trim_start_matches('\u{feff}');
    let Ok(value) = serde_yaml::from_str::<Value>(source) else {
        return Vec::new();
    };
    let located = step_run_lines(source, action);
    let mut findings = Vec::new();
    for body in bodies(action, &value) {
        let lines = located
            .iter()
            .find(|item| item.job == body.job && item.step == body.step);
        findings.extend(judge(rel, &body, lines));
    }
    findings
}

fn bodies(action: bool, value: &Value) -> Vec<Body> {
    if action {
        return composite_bodies(value);
    }
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, job) in jobs {
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let Some(steps) = mapping_get(job, "steps").and_then(Value::as_sequence) else {
            continue;
        };
        push_runs(&scalar_name(id), steps, false, &mut out);
    }
    out
}

fn composite_bodies(value: &Value) -> Vec<Body> {
    if value
        .get("runs")
        .and_then(|runs| runs.get("using"))
        .and_then(Value::as_str)
        != Some("composite")
    {
        return Vec::new();
    }
    let Some(steps) = value
        .get("runs")
        .and_then(|runs| runs.get("steps"))
        .and_then(Value::as_sequence)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    push_runs("", steps, true, &mut out);
    out
}

fn push_runs(job: &str, steps: &[Value], composite: bool, out: &mut Vec<Body>) {
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(script) = mapping_get(step, "run").and_then(Value::as_str) else {
            continue;
        };
        out.push(Body {
            job: if composite {
                String::new()
            } else {
                job.to_string()
            },
            step: index + 1,
            script: script.to_string(),
            composite,
            target: if composite {
                "composite".to_string()
            } else {
                job.to_string()
            },
        });
    }
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
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line,
                message: format!(
                    "{rel}:{line}: {place} pins host port `{host}` in `{spec}`. \
                     Hosted and shared runners collide when a docker run publish mapping \
                     fixes the host port. Publish a Docker-allocated port \
                     (`127.0.0.1::5432`, `::5432`, or the container port only) or a \
                     variable host port (`$PORT:5432`, `${{PORT}}:5432`, `${{{{ ... }}}}:5432`).",
                    host = hit.host,
                    spec = hit.spec,
                ),
                import: None,
                target: Some(body.target.clone()),
            }
        })
        .collect()
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

fn scalar_name(key: &Value) -> String {
    match key {
        Value::String(name) if !name.is_empty() => name.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => "job".to_string(),
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}
