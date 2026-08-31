pub(super) fn checkout_key_line(source: &str, key: &str, occurrence: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let code_lines = yaml_code_lines(&lines);
    let checkout_line = code_lines
        .iter()
        .enumerate()
        .filter(|(index, line)| checkout_reference_on_line(&lines, &code_lines, *index, line))
        .nth(occurrence)
        .map(|(index, _)| index)
        .unwrap_or(0);
    if flow_step_has_key(code_lines[checkout_line], key) {
        return checkout_line + 1;
    }
    let step_start =
        checkout_step_start(&lines, &code_lines, checkout_line).unwrap_or(checkout_line);
    let step_indent = leading_indent(lines[step_start]);
    let mut with_indent = None;
    for (index, line) in code_lines.iter().enumerate().skip(step_start) {
        let indent = leading_indent(lines[index]);
        let trimmed = line.trim();
        if index > step_start && is_sequence_entry(trimmed) && indent <= step_indent {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        if with_indent.is_none() && yaml_key_value(trimmed, "with").is_some() {
            with_indent = Some(indent);
            continue;
        }
        let Some(with_indent) = with_indent else {
            continue;
        };
        if !trimmed.is_empty() && indent <= with_indent {
            break;
        }
        if yaml_key_line(trimmed, key) {
            return index + 1;
        }
    }
    checkout_line + 1
}

fn yaml_code_lines<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    let mut block_scalar_indent = None;
    lines
        .iter()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return "";
            }
            let indent = leading_indent(line);
            if block_scalar_indent.is_some_and(|block_indent| indent > block_indent) {
                return "";
            }
            block_scalar_indent = None;
            if is_block_scalar_header(trimmed) {
                // A key on a sequence entry starts after `- `, so sibling mapping
                // keys may be indented beyond the dash without belonging to the scalar.
                block_scalar_indent = Some(if line.trim_start().starts_with("- ") {
                    indent + 2
                } else {
                    indent
                });
            }
            *line
        })
        .collect()
}

fn is_block_scalar_header(line: &str) -> bool {
    let line = line.strip_prefix("- ").map(str::trim_start).unwrap_or(line);
    line.split_once(':')
        .is_some_and(|(_, value)| matches!(value.trim_start().chars().next(), Some('|' | '>')))
}

fn checkout_reference_on_line(
    lines: &[&str],
    code_lines: &[&str],
    index: usize,
    line: &str,
) -> bool {
    let Some(step_start) = checkout_step_start(lines, code_lines, index) else {
        return false;
    };
    if index == step_start
        && flow_step_mapping(line).is_some_and(|mapping| {
            mapping
                .get("uses")
                .and_then(serde_yaml::Value::as_str)
                .is_some_and(is_checkout_reference)
        })
    {
        return true;
    }
    mapping_key_indent(lines[index]) == leading_indent(lines[step_start]) + 2
        && yaml_key_value(line.trim(), "uses")
            .and_then(|value| yaml_string_value(lines, index, value))
            .is_some_and(|uses| is_checkout_reference(&uses))
}

fn checkout_step_start(lines: &[&str], code_lines: &[&str], checkout_line: usize) -> Option<usize> {
    let checkout_indent = leading_indent(lines[checkout_line]);
    let step_start = (0..=checkout_line).rev().find(|index| {
        let line = code_lines[*index].trim_start();
        is_sequence_entry(line) && leading_indent(lines[*index]) <= checkout_indent
    })?;
    let step_indent = leading_indent(lines[step_start]);
    (0..step_start)
        .rev()
        .find(|index| {
            let line = code_lines[*index].trim();
            let indent = leading_indent(lines[*index]);
            !line.is_empty()
                && indent <= step_indent
                && (indent < step_indent || !is_sequence_entry(line))
        })
        .filter(|steps_key| yaml_key_value(code_lines[*steps_key].trim(), "steps").is_some())
        .map(|_| step_start)
}

fn flow_step_has_key(line: &str, key: &str) -> bool {
    flow_step_mapping(line)
        .and_then(|mapping| mapping.get("with")?.as_mapping().cloned())
        .is_some_and(|with| with.contains_key(serde_yaml::Value::String(key.to_string())))
}

fn flow_step_mapping(line: &str) -> Option<serde_yaml::Mapping> {
    let value = line.trim_start().strip_prefix("- ")?.trim_start();
    value
        .starts_with('{')
        .then(|| serde_yaml::from_str(value).ok())
        .flatten()
}

fn yaml_string_value(lines: &[&str], index: usize, value: &str) -> Option<String> {
    if matches!(value.trim_start().chars().next(), Some('|' | '>')) {
        let header_indent = leading_indent(lines[index]);
        let mut document = format!("uses: {value}\n");
        for line in lines.iter().skip(index + 1) {
            if !line.trim().is_empty() && leading_indent(line) <= header_indent {
                break;
            }
            document.push_str(line);
            document.push('\n');
        }
        return serde_yaml::from_str::<serde_yaml::Mapping>(&document)
            .ok()
            .and_then(|mapping| {
                mapping
                    .get("uses")
                    .and_then(serde_yaml::Value::as_str)
                    .map(str::to_string)
            });
    }
    serde_yaml::from_str::<String>(value).ok()
}

pub(super) fn is_checkout_reference(uses: &str) -> bool {
    const PREFIX: &str = "actions/checkout@";
    uses.trim()
        .get(..PREFIX.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(PREFIX))
}

fn is_sequence_entry(line: &str) -> bool {
    line == "-" || line.starts_with("- ")
}

fn leading_indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn mapping_key_indent(line: &str) -> usize {
    leading_indent(line) + usize::from(line.trim_start().starts_with("- ")) * 2
}

pub(super) fn yaml_key_line(line: &str, key: &str) -> bool {
    yaml_key_value(line, key).is_some()
}

fn yaml_key_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let line = line.strip_prefix("- ").map(str::trim_start).unwrap_or(line);
    for prefix in [
        format!("{key}:"),
        format!("'{key}':"),
        format!("\"{key}\":"),
    ] {
        if let Some(value) = line.strip_prefix(&prefix) {
            return Some(value.trim_start());
        }
    }
    None
}
