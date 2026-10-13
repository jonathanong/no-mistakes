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
    let minutes = value.as_u64()?;
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
    let mut jobs_indent = None;
    let mut job_key_indent = None;
    let mut job_indent = None;
    let mut steps_indent = None;
    let mut found_steps = false;
    let mut item_indent = None;
    let mut seen = 0;
    for (line_number, line) in source.lines().enumerate() {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let trimmed = line.trim_start();
        let key = trimmed.split('#').next().unwrap_or(trimmed).trim_end();
        if jobs_indent.is_none() {
            if indent == 0 && key == "jobs:" {
                jobs_indent = Some(indent);
            }
            continue;
        }
        if let Some(depth) = job_indent {
            if indent <= depth && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        } else {
            if indent <= jobs_indent.unwrap() && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some(candidate) = key.strip_suffix(':') else {
                continue;
            };
            let depth = *job_key_indent.get_or_insert(indent);
            if indent == depth && candidate.trim_matches(['\'', '"']) == job_id {
                job_indent = Some(indent);
            }
            continue;
        }
        if !found_steps && key == "steps:" {
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

pub(super) fn step_line_path(source: &str, job_id: &str, path: &[usize]) -> Option<usize> {
    let (first, rest) = path.split_first()?;
    let mut line = step_line(source, job_id, *first)?;
    for index in rest {
        line = parallel_child_line(source, line, *index)?;
    }
    Some(line)
}

fn parallel_child_line(source: &str, parent_line: usize, index: usize) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let parent = *lines.get(parent_line.checked_sub(1)?)?;
    let parent_indent = parent.len() - parent.trim_start_matches(' ').len();
    let inline = parent.trim_start().starts_with("- parallel:");
    let mut key = inline.then_some((parent_line - 1, parent_indent + 2));
    if key.is_none() {
        for (line_index, line) in lines.iter().enumerate().skip(parent_line) {
            let indent = line.len() - line.trim_start_matches(' ').len();
            let trimmed = line.trim_start();
            if indent <= parent_indent && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            if trimmed.split('#').next().unwrap_or(trimmed).trim_end() == "parallel:" {
                key = Some((line_index, indent));
                break;
            }
        }
    }
    let (key_line, key_indent) = key?;
    let mut item_indent = None;
    let mut seen = 0;
    for (line_index, line) in lines.iter().enumerate().skip(key_line + 1) {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let trimmed = line.trim_start();
        let is_item = trimmed.starts_with("- ") || trimmed == "-";
        if (indent < key_indent || (indent == key_indent && !is_item))
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            break;
        }
        if indent >= key_indent && is_item {
            let depth = *item_indent.get_or_insert(indent);
            if indent == depth {
                if seen == index {
                    return Some(line_index + 1);
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
