use serde_yaml::{Mapping, Value};

pub(super) struct Body<'a> {
    pub(super) job: String,
    pub(super) step: usize,
    pub(super) script: &'a str,
    pub(super) env: Option<&'a Mapping>,
    pub(super) target: String,
}

pub(super) fn bodies(action: bool, value: &Value) -> Vec<Body<'_>> {
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
        let shell = default_shell(mapping_get(job, "defaults"))
            .or_else(|| default_shell(value.get("defaults")));
        let assume_bourne = !mapping_get(job, "runs-on").is_some_and(unknown_or_windows_runner);
        push_runs(&scalar_name(id), steps, shell, assume_bourne, &mut out);
    }
    out
}

fn composite_bodies(value: &Value) -> Vec<Body<'_>> {
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
    push_runs("", steps, None, true, &mut out);
    out
}

fn push_runs<'a>(
    job: &str,
    steps: &'a [Value],
    default_shell: Option<&str>,
    assume_bourne: bool,
    out: &mut Vec<Body<'a>>,
) {
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(script) = mapping_get(step, "run").and_then(Value::as_str) else {
            continue;
        };
        let target = if job.is_empty() {
            "composite".to_string()
        } else {
            job.to_string()
        };
        let shell = mapping_get(step, "shell")
            .and_then(Value::as_str)
            .or(default_shell);
        out.push(Body {
            job: job.to_string(),
            step: index + 1,
            script,
            env: mapping_get(step, "env")
                .and_then(Value::as_mapping)
                .filter(|_| shell.map_or(assume_bourne, is_bourne_shell)),
            target,
        });
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}

fn scalar_name(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => "job".to_string(),
    }
}

fn default_shell(defaults: Option<&Value>) -> Option<&str> {
    defaults?.get("run")?.get("shell")?.as_str()
}

fn is_bourne_shell(template: &str) -> bool {
    let Some(program) = template.split_whitespace().next() else {
        return false;
    };
    matches!(
        program.rsplit('/').next().unwrap_or(program),
        "bash" | "sh" | "dash" | "ash" | "zsh" | "ksh"
    )
}

fn unknown_or_windows_runner(labels: &Value) -> bool {
    if let Some(label) = labels.as_str() {
        let label = label.to_ascii_lowercase();
        return label == "windows" || label.starts_with("windows-") || label.contains("${{");
    }
    if let Some(labels) = labels.as_sequence() {
        return labels.iter().any(unknown_or_windows_runner);
    }
    labels.get("labels").is_some_and(unknown_or_windows_runner)
        || labels
            .get("group")
            .and_then(Value::as_str)
            .is_some_and(|group| group.contains("${{"))
}
