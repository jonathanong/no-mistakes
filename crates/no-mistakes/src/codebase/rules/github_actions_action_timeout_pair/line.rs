use super::yaml::{finding, literal_u64, mapping_get, nested_message, yaml_got};
use super::{CompiledOptions, RuleFinding};
use serde_yaml::{Mapping, Value};

pub(super) fn nested_input_finding(
    rel: &str,
    source: &str,
    job_id: &str,
    label: &str,
    step: &Mapping,
    index: usize,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    if opts.nested_input.is_empty() {
        return Vec::new();
    }
    let Some(expected) = opts.nested_timeout_seconds else {
        return Vec::new();
    };
    let raw = mapping_get(step, "with")
        .and_then(Value::as_mapping)
        .and_then(|with| mapping_get(with, &opts.nested_input));
    if raw.and_then(literal_u64) == Some(expected) {
        return Vec::new();
    }
    vec![finding(
        rel,
        nested_timeout_line(source, job_id, index, &opts.nested_input),
        nested_message(
            rel,
            job_id,
            label,
            &opts.nested_input,
            expected,
            &yaml_got(raw),
        ),
        job_id,
    )]
}

pub(super) fn nested_timeout_line(
    source: &str,
    job_id: &str,
    index: usize,
    nested_input: &str,
) -> usize {
    step_key_line_in(source, job_id, index, &[nested_input, "uses", "with"])
}

pub(super) fn step_key_line(source: &str, job_id: &str, index: usize, key: &str) -> usize {
    step_key_line_in(source, job_id, index, &[key, "uses"])
}

fn step_key_line_in(source: &str, job_id: &str, index: usize, keys: &[&str]) -> usize {
    let Some(start) = step_start_line(source, job_id, index) else {
        return 1;
    };
    let end = next_step_end(source, start);
    keys.iter()
        .find_map(|key| key_between(source, start, end, key))
        .unwrap_or(start)
}

fn step_start_line(source: &str, job_id: &str, index: usize) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let from = if job_id == "(composite)" {
        0
    } else {
        lines
            .iter()
            .position(|line| is_key(trim_comment(line), job_id))?
    };
    let steps_at = lines
        .iter()
        .enumerate()
        .skip(from)
        .find_map(|(offset, line)| is_key(trim_comment(line), "steps").then_some(offset))?;
    let steps_indent = indent(lines[steps_at]);
    let mut step_indent = None;
    let mut seen = 0usize;
    for (offset, line) in lines.iter().enumerate().skip(steps_at + 1) {
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent(line);
        if indent <= steps_indent {
            break;
        }
        if !trim_comment(line).starts_with("- ") {
            continue;
        }
        match step_indent {
            None => step_indent = Some(indent),
            Some(expected) if indent != expected => continue,
            Some(_) => {}
        }
        if seen == index {
            return Some(offset + 1);
        }
        seen += 1;
    }
    None
}

fn next_step_end(source: &str, start: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let start_idx = start.saturating_sub(1);
    let start_indent = indent(lines.get(start_idx).copied().unwrap_or(""));
    lines
        .iter()
        .enumerate()
        .skip(start_idx + 1)
        .find_map(|(offset, line)| {
            if line.trim().is_empty() {
                return None;
            }
            let indent = indent(line);
            let trimmed = trim_comment(line);
            ((trimmed.starts_with("- ") && indent <= start_indent) || indent < start_indent)
                .then_some(offset + 1)
        })
        .unwrap_or(lines.len() + 1)
}

fn key_between(source: &str, start: usize, end: usize, key: &str) -> Option<usize> {
    source
        .lines()
        .enumerate()
        .take(end.saturating_sub(1))
        .skip(start.saturating_sub(1))
        .find_map(|(offset, line)| {
            let trimmed = trim_comment(line);
            let body = trimmed.strip_prefix("- ").unwrap_or(trimmed);
            is_key(body, key).then_some(offset + 1)
        })
}

fn is_key(trimmed: &str, key: &str) -> bool {
    let prefix = format!("{key}:");
    trimmed == prefix || trimmed.starts_with(&prefix)
}

fn trim_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or(line).trim()
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}
