use serde_yaml::{Mapping, Value};

pub(in crate::codebase::rules) struct RunScript {
    pub(in crate::codebase::rules) job: String,
    pub(in crate::codebase::rules) step: usize,
    pub(in crate::codebase::rules) script: String,
    pub(in crate::codebase::rules) composite: bool,
}

pub(in crate::codebase::rules) fn run_scripts(action: bool, value: &Value) -> Vec<RunScript> {
    if action {
        return composite_scripts(value);
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
        push_scripts(&scalar_name(id), steps, false, &mut out);
    }
    out
}

fn composite_scripts(value: &Value) -> Vec<RunScript> {
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
    push_scripts("", steps, true, &mut out);
    out
}

fn push_scripts(job: &str, steps: &[Value], composite: bool, out: &mut Vec<RunScript>) {
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let Some(script) = mapping_get(step, "run").and_then(Value::as_str) else {
            continue;
        };
        out.push(RunScript {
            job: if composite {
                String::new()
            } else {
                job.to_string()
            },
            step: index + 1,
            script: script.to_string(),
            composite,
        });
    }
}

pub(in crate::codebase::rules) fn scalar_name(key: &Value) -> String {
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
