use super::{RuleFinding, RULE_ID};
use serde_yaml::{Mapping, Value};

pub(super) fn has_key(map: &Mapping, key: &str) -> bool {
    map.contains_key(Value::String(key.into()))
}
pub(super) fn mapping_str<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}
pub(super) fn timeout_minutes(value: &Value) -> Option<u64> {
    let minutes = match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }?;
    (minutes > 0).then_some(minutes)
}

pub(super) fn is_expression(value: &Value) -> bool {
    value.as_str().is_some_and(|text| {
        let text = text.trim();
        text.starts_with("${{") && text.ends_with("}}")
    })
}

// Locate the sequence entry instead of the first timeout key, so line suppressions
// work on workflows with several steps.
pub(super) fn step_line(source: &str, job_id: &str, index: usize) -> Option<usize> {
    let mut job_indent = None;
    let mut steps_indent = None;
    let mut found_steps = false;
    let mut item_indent = None;
    let mut seen = 0;
    for (line_number, line) in source.lines().enumerate() {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let trimmed = line.trim_start();
        if let Some(depth) = job_indent {
            if indent <= depth && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        } else if trimmed.trim_end_matches(':').trim_matches(['\'', '"']) == job_id
            && trimmed.ends_with(':')
        {
            job_indent = Some(indent);
            continue;
        }
        if job_indent.is_some() && !found_steps && trimmed == "steps:" {
            steps_indent = Some(indent);
            found_steps = true;
            continue;
        }
        if let Some(depth) = steps_indent {
            let is_item = trimmed.starts_with("- ") || trimmed == "-";
            if (indent < depth || (indent == depth && !is_item))
                && !trimmed.is_empty()
                && !trimmed.starts_with('#')
            {
                steps_indent = None;
                continue;
            }
            if indent >= depth && is_item {
                let item_depth = *item_indent.get_or_insert(indent);
                if indent != item_depth {
                    continue;
                }
                if seen == index {
                    return Some(line_number + 1);
                }
                seen += 1;
            }
        }
    }
    None
}

pub(super) fn finding(
    file: &str,
    line: usize,
    message: String,
    target: Option<&str>,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: target.map(str::to_owned),
    }
}
