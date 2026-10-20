use super::mapping::{classify, Verdict};
use super::publish::publish_hits;
use super::runs::pin_message;
use super::{RuleFinding, RULE_ID};
use serde_yaml::{Mapping, Value};

/// Service and container ports come from the parsed workflow. A block sequence
/// may sit in the same column as `ports:`, and a flow mapping is not a line
/// of its own. The line scan only places the spec that the value already judged.
pub(super) fn service_findings(rel: &str, source: &str, jobs: &Mapping) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (id, job) in jobs {
        let Some(job) = job.as_mapping() else {
            continue;
        };
        let name = scalar_name(id);
        if let Some(services) = map_get(job, "services").and_then(Value::as_mapping) {
            for (service_id, service) in services {
                let Some(service) = service.as_mapping() else {
                    continue;
                };
                let place = format!("job `{name}` service `{}`", scalar_name(service_id));
                note_ports(
                    &mut findings,
                    rel,
                    source,
                    &name,
                    &place,
                    map_get(service, "ports"),
                );
                note_options(
                    &mut findings,
                    rel,
                    source,
                    &name,
                    &place,
                    map_get(service, "options"),
                );
            }
        }
        if let Some(container) = map_get(job, "container").and_then(Value::as_mapping) {
            let place = format!("job `{name}` container");
            note_ports(
                &mut findings,
                rel,
                source,
                &name,
                &place,
                map_get(container, "ports"),
            );
        }
    }
    findings
}

fn note_ports(
    findings: &mut Vec<RuleFinding>,
    rel: &str,
    source: &str,
    job: &str,
    place: &str,
    ports: Option<&Value>,
) {
    let Some(ports) = ports else {
        return;
    };
    for spec in port_specs(ports) {
        note(findings, rel, source, job, place, &spec);
    }
}

fn note_options(
    findings: &mut Vec<RuleFinding>,
    rel: &str,
    source: &str,
    job: &str,
    place: &str,
    options: Option<&Value>,
) {
    let Some(text) = options.and_then(Value::as_str) else {
        return;
    };
    for hit in publish_hits(&format!("docker run {text} img")) {
        note(findings, rel, source, job, place, &hit.spec);
    }
}

fn note(
    findings: &mut Vec<RuleFinding>,
    rel: &str,
    source: &str,
    job: &str,
    place: &str,
    spec: &str,
) {
    let Verdict::Literal(host) = classify(spec) else {
        return;
    };
    let line = spec_line(source, job, spec);
    findings.push(RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: pin_message(rel, line, place, &host, spec),
        import: None,
        target: Some(job.to_string()),
    });
}

fn port_specs(value: &Value) -> Vec<String> {
    match value {
        Value::Sequence(items) => items.iter().filter_map(scalar_spec).collect(),
        other => scalar_spec(other).into_iter().collect(),
    }
}

fn scalar_spec(value: &Value) -> Option<String> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_string()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn spec_line(source: &str, job: &str, spec: &str) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let (start, end) = job_span(&lines, job).unwrap_or((0, lines.len()));
    lines[start..end]
        .iter()
        .position(|line| line.contains(spec))
        .map(|offset| start + offset + 1)
        .unwrap_or(start + 1)
}

/// Lines belonging to one job, so a repeated spec is not taken from another job.
fn job_span(lines: &[&str], job: &str) -> Option<(usize, usize)> {
    let jobs_at = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed == "jobs:" || trimmed.starts_with("jobs:")
    })?;
    let jobs_indent = indent(lines[jobs_at]);
    let child = lines.iter().skip(jobs_at + 1).find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        let level = indent(line);
        (level > jobs_indent).then_some(level)
    })?;
    let mut start = None;
    for (index, line) in lines.iter().enumerate().skip(jobs_at + 1) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let level = indent(line);
        if level < child {
            break;
        }
        if level == child && key_is(trimmed, job) {
            if let Some(found) = start {
                return Some((found, index));
            }
            start = Some(index);
        }
    }
    start.map(|found| (found, lines.len()))
}

fn key_is(trimmed: &str, job: &str) -> bool {
    let Some((key, _)) = trimmed.split_once(':') else {
        return false;
    };
    key.trim().trim_matches(['"', '\'']) == job
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn scalar_name(key: &Value) -> String {
    match key {
        Value::String(name) if !name.is_empty() => name.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => "job".to_string(),
    }
}

fn map_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}
