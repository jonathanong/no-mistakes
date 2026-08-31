mod steps;

pub(super) fn checkout_key_line(source: &str, key: &str, occurrence: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let code_lines = yaml_code_lines(&lines);
    let line = steps::checkout_steps(&lines, &code_lines)
        .nth(occurrence)
        .map_or(1, |step| step.key_line(&lines, &code_lines, key));
    line
}

fn yaml_code_lines<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    let mut block_scalar_indent = None;
    lines
        .iter()
        .map(|line| {
            let trimmed = line.trim();
            let indent = leading_indent(line);
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || block_scalar_indent.is_some_and(|block_indent| indent > block_indent)
            {
                return "";
            }
            block_scalar_indent = is_block_scalar_header(trimmed)
                .then_some(indent + usize::from(line.trim_start().starts_with("- ")) * 2);
            *line
        })
        .collect()
}

fn is_block_scalar_header(line: &str) -> bool {
    let line = step_line(line, true);
    line.split_once(':')
        .is_some_and(|(_, value)| matches!(value.trim_start().chars().next(), Some('|' | '>')))
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

fn step_line(line: &str, is_start: bool) -> &str {
    let line = line.trim_start();
    if is_start {
        line.strip_prefix('-').unwrap_or(line).trim_start()
    } else {
        line
    }
}

pub(super) fn yaml_key_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let line = step_line(line, true);
    let (candidate, value) = line.split_once(':')?;
    matches!(candidate.trim_end(), candidate_key if candidate_key == key
        || candidate_key == format!("'{key}'")
        || candidate_key == format!("\"{key}\""))
    .then(|| value.trim_start())
}
