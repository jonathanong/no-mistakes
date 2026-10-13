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

fn yaml_key_value(line: &str) -> Option<(&str, &str)> {
    let text = line.trim_start();
    let text = text.strip_prefix("- ").unwrap_or(text);
    let (key, value) = text.split_once(':')?;
    let key = key.trim().trim_matches(['\'', '"']);
    (!key.is_empty() && !key.starts_with('#')).then_some((key, value.trim()))
}

fn is_inline_value_line(line: &str) -> bool {
    yaml_key_value(line).is_some_and(|(_, value)| value.starts_with(&['*', '[', '{'][..]))
}

// Locate the sequence entry instead of the first timeout key, so line suppressions
// work on workflows with several steps.
pub(super) fn step_line(source: &str, job_id: &str, index: usize) -> Option<usize> {
    let mut jobs_indent = None;
    let mut job_key_indent = None;
    let mut job_indent = None;
    let mut job_child_indent = None;
    let mut steps_indent = None;
    let mut found_steps = false;
    let mut item_indent = None;
    let mut seen = 0;
    for (line_number, line) in source.lines().enumerate() {
        let indent = line.len() - line.trim_start_matches(' ').len();
        let trimmed = line.trim_start();
        if jobs_indent.is_none() {
            if indent == 0 && yaml_key_value(line).is_some_and(|(key, _)| key == "jobs") {
                if is_inline_value_line(line) {
                    return Some(line_number + 1);
                }
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
            let Some((candidate, _)) = yaml_key_value(line) else {
                continue;
            };
            let depth = *job_key_indent.get_or_insert(indent);
            if indent == depth && candidate == job_id {
                if is_inline_value_line(line) {
                    return Some(line_number + 1);
                }
                job_indent = Some(indent);
            }
            continue;
        }
        if job_child_indent.is_none()
            && indent > job_indent.unwrap()
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            job_child_indent = Some(indent);
        }
        if !found_steps
            && indent == job_child_indent.unwrap_or(usize::MAX)
            && yaml_key_value(line).is_some_and(|(key, _)| key == "steps")
        {
            if is_inline_value_line(line) {
                return Some(line_number + 1);
            }
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
    if is_inline_value_line(source.lines().nth(line - 1)?) {
        return Some(line);
    }
    for index in rest {
        line = parallel_child_line(source, line, *index)?;
    }
    Some(line)
}

fn parallel_child_line(source: &str, parent_line: usize, index: usize) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();
    let parent = *lines.get(parent_line.checked_sub(1)?)?;
    let parent_indent = parent.len() - parent.trim_start_matches(' ').len();
    let inline = parent.trim_start().starts_with("- ")
        && yaml_key_value(parent).is_some_and(|(key, _)| key == "parallel");
    let mut key = inline.then_some((parent_line - 1, parent_indent + 2));
    if key.is_none() {
        for (line_index, line) in lines.iter().enumerate().skip(parent_line) {
            let indent = line.len() - line.trim_start_matches(' ').len();
            let trimmed = line.trim_start();
            if indent <= parent_indent && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            if yaml_key_value(line).is_some_and(|(key, _)| key == "parallel") {
                key = Some((line_index, indent));
                break;
            }
        }
    }
    let (key_line, key_indent) = key?;
    if is_inline_value_line(lines[key_line]) {
        return Some(key_line + 1);
    }
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
