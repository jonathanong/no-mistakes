pub(super) fn checkout_key_line(source: &str, key: &str, occurrence: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let code_lines = yaml_code_lines(&lines);
    let checkout_line = code_lines
        .iter()
        .enumerate()
        .filter(|(_, line)| checkout_reference_on_line(line))
        .nth(occurrence)
        .map(|(index, _)| index)
        .unwrap_or(0);
    let checkout_indent = leading_indent(lines[checkout_line]);
    let step_start = (0..=checkout_line)
        .rev()
        .find(|index| {
            let line = code_lines[*index].trim_start();
            is_sequence_entry(line) && leading_indent(lines[*index]) <= checkout_indent
        })
        .unwrap_or(checkout_line);
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
                block_scalar_indent = Some(indent);
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

fn checkout_reference_on_line(line: &str) -> bool {
    yaml_key_value(line.trim(), "uses")
        .and_then(|value| serde_yaml::from_str::<String>(value).ok())
        .is_some_and(|uses| is_checkout_reference(&uses))
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
