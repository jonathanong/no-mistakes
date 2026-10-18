use super::expressions::expressions;
use super::hit::Hit;
use super::yaml_text::is_github_script;
use serde_yaml::Value;

pub(super) fn is_composite(value: &Value) -> bool {
    value
        .get("runs")
        .and_then(|runs| runs.get("using"))
        .and_then(Value::as_str)
        == Some("composite")
}

pub(super) fn hits_from_value(value: &Value, action: bool) -> Vec<Hit> {
    let mut hits = Vec::new();
    if action {
        if is_composite(value) {
            if let Some(steps) = value
                .get("runs")
                .and_then(|runs| runs.get("steps"))
                .and_then(Value::as_sequence)
            {
                collect_steps(steps, &mut hits);
            }
        }
        return hits;
    }
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return hits;
    };
    for (_, job) in jobs {
        if let Some(steps) = job.get("steps").and_then(Value::as_sequence) {
            collect_steps(steps, &mut hits);
        }
    }
    hits
}

fn collect_steps(steps: &[Value], hits: &mut Vec<Hit>) {
    for step in steps {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        if let Some(run) = mapping_str(step, "run") {
            push_exprs(hits, run, false);
        }
        if mapping_str(step, "uses").is_some_and(is_github_script) {
            if let Some(script) = step
                .get("with")
                .and_then(|with| with.get("script"))
                .and_then(Value::as_str)
            {
                push_exprs(hits, script, true);
            }
        }
    }
}

fn mapping_str<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    map.get(Value::String(key.to_string()))
        .and_then(Value::as_str)
}

fn push_exprs(hits: &mut Vec<Hit>, text: &str, script: bool) {
    for expr in expressions(text) {
        hits.push(Hit {
            line: 0,
            expr,
            script,
        });
    }
}
