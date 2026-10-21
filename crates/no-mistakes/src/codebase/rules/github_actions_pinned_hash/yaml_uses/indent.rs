pub(super) fn in_input_block(lines: &[&str], index: usize) -> bool {
    let own = indent_of(lines.get(index).copied().unwrap_or(""));
    for prev in (0..index).rev() {
        let trimmed = key_text(lines[prev]);
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if indent_of(lines[prev]) < own {
            return trimmed.starts_with("with:") || trimmed.starts_with("env:");
        }
    }
    false
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
