use super::{CiEnvLocation, EnvLocationKind, EnvScope};
use regex::Regex;
use serde_yaml::Value;

pub(super) fn reference_regex(var: &str) -> Regex {
    // `env` must be a standalone context, not a property segment (e.g.
    // `github.event.inputs.env.FOO`). Match dotted and indexed access.
    let v = regex::escape(var);
    let pattern = [
        r#"\$\{\{(?:[^}]*[^.\w])?env(?:\."#,
        &v,
        r#"\b|\['"#,
        &v,
        r#"'\]|\[""#,
        &v,
        r#""\])[^}]*\}\}"#,
    ]
    .concat();
    Regex::new(&pattern).expect("env reference regex is well-formed")
}

pub(super) fn collect_locations(
    value: &Value,
    var: &str,
    reference_re: &Regex,
) -> Vec<CiEnvLocation> {
    let mut out = Vec::new();
    let Some(root_map) = value.as_mapping() else {
        return out;
    };
    collect_env_defs(root_map, EnvScope::Workflow, None, var, &mut out);
    for (key, val) in root_map {
        if key.as_str() != Some("jobs") {
            scan_refs(val, EnvScope::Workflow, None, reference_re, &mut out);
        }
    }
    if let Some(jobs) = root_map.get("jobs").and_then(Value::as_mapping) {
        collect_job_locations(jobs, var, reference_re, &mut out);
    }
    sort_locations(out)
}

fn collect_job_locations(
    jobs: &serde_yaml::Mapping,
    var: &str,
    reference_re: &Regex,
    out: &mut Vec<CiEnvLocation>,
) {
    for (job_key, job_val) in jobs {
        let job_id = job_key.as_str().unwrap_or_default().to_string();
        let Some(job_map) = job_val.as_mapping() else {
            continue;
        };
        collect_env_defs(job_map, EnvScope::Job, Some(&job_id), var, out);
        for (key, val) in job_map {
            if key.as_str() != Some("steps") {
                scan_refs(val, EnvScope::Job, Some(&job_id), reference_re, out);
            }
        }
        if let Some(steps) = job_map.get("steps").and_then(Value::as_sequence) {
            for step in steps {
                if let Some(step_map) = step.as_mapping() {
                    collect_env_defs(step_map, EnvScope::Step, Some(&job_id), var, out);
                }
                scan_refs(step, EnvScope::Step, Some(&job_id), reference_re, out);
            }
        }
    }
}

fn collect_env_defs(
    map: &serde_yaml::Mapping,
    scope: EnvScope,
    job: Option<&str>,
    var: &str,
    out: &mut Vec<CiEnvLocation>,
) {
    let Some(env) = map.get("env").and_then(Value::as_mapping) else {
        return;
    };
    for (key, val) in env {
        if key.as_str() == Some(var) {
            out.push(CiEnvLocation {
                kind: EnvLocationKind::Definition,
                scope,
                job: job.map(str::to_string),
                value: scalar_to_string(val),
            });
        }
    }
}

fn scan_refs(
    value: &Value,
    scope: EnvScope,
    job: Option<&str>,
    reference_re: &Regex,
    out: &mut Vec<CiEnvLocation>,
) {
    match value {
        Value::String(s) if reference_re.is_match(s) => out.push(CiEnvLocation {
            kind: EnvLocationKind::Reference,
            scope,
            job: job.map(str::to_string),
            value: None,
        }),
        Value::Sequence(seq) => {
            for item in seq {
                scan_refs(item, scope, job, reference_re, out);
            }
        }
        Value::Mapping(map) => {
            for val in map.values() {
                scan_refs(val, scope, job, reference_re, out);
            }
        }
        _ => {}
    }
}

pub(super) fn scalar_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Keep duplicate references because they are distinct audited occurrences.
fn sort_locations(mut locations: Vec<CiEnvLocation>) -> Vec<CiEnvLocation> {
    locations.sort_by(|a, b| {
        (a.kind, a.scope, &a.job, &a.value).cmp(&(b.kind, b.scope, &b.job, &b.value))
    });
    locations
}
