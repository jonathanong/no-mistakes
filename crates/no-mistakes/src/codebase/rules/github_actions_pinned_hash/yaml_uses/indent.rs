pub(super) fn in_input_block(lines: &[&str], index: usize) -> bool {
    let own = indent_of(lines.get(index).copied().unwrap_or(""));
    let Some(parent) = less_indented(lines, index, own) else {
        return false;
    };
    if !is_input_key(key_text(lines[parent])) {
        return false;
    }
    // A job id may be `env` or `with`. That key sits directly under `jobs:`.
    let parent_indent = indent_of(lines[parent]);
    match less_indented(lines, parent, parent_indent) {
        Some(grand) => !key_text(lines[grand]).starts_with("jobs:"),
        None => true,
    }
}

fn less_indented(lines: &[&str], index: usize, indent: usize) -> Option<usize> {
    (0..index).rev().find(|&prev| {
        let trimmed = key_text(lines[prev]);
        !trimmed.is_empty() && !trimmed.starts_with('#') && indent_of(lines[prev]) < indent
    })
}

fn is_input_key(trimmed: &str) -> bool {
    trimmed.starts_with("with:") || trimmed.starts_with("env:")
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn key_text(line: &str) -> &str {
    line.trim().trim_start_matches('-').trim()
}

pub(super) fn is_value_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && !trimmed.starts_with(['-', '#'])
}

pub(super) fn is_ignorable(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// A nested mapping entry (`key:` / `key: value`). `docker://image` is a scalar:
/// its colon is followed by `/`, not by a break.
pub(super) fn is_nested_key(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(['#', '-']) {
        return false;
    }
    let Some(index) = trimmed.find(':') else {
        return false;
    };
    let after = &trimmed[index + 1..];
    after.is_empty() || after.starts_with([' ', '\t', '#'])
}

#[cfg(test)]
#[path = "indent/tests.rs"]
mod tests;
