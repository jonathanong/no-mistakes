pub(crate) fn yaml_parse_line(err: &serde_yaml::Error) -> usize {
    err.location().map_or(1, |location| location.line())
}

pub(crate) fn mapping_key_line(source: &str, key: &str, occurrence: usize) -> usize {
    mapping_key_candidate_lines(source, key)
        .nth(occurrence)
        .unwrap_or(1)
}

/// Mapping-key lines that are not inside a `|` / `>` block scalar.
///
/// A block-scalar body can contain `timeout-minutes:` as documentation. Those
/// lines must not steal the occurrence index of a later real sibling key.
fn mapping_key_candidate_lines<'a>(
    source: &'a str,
    key: &'a str,
) -> impl Iterator<Item = usize> + 'a {
    let mut block_indent = None;
    source.lines().enumerate().filter_map(move |(index, line)| {
        let indent = leading_indent(line);
        let trimmed = line.trim();
        if let Some(key_indent) = block_indent {
            if trimmed.is_empty() || trimmed.starts_with('#') || indent > key_indent {
                return None;
            }
            block_indent = None;
        }
        if starts_block_scalar(line) {
            block_indent = Some(indent);
        }
        contains_mapping_key(line, key).then_some(index + 1)
    })
}

fn leading_indent(line: &str) -> usize {
    line.chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .count()
}

pub(crate) fn starts_block_scalar(line: &str) -> bool {
    let comment_free = line.split('#').next().unwrap_or(line);
    let Some(colon) = comment_free.find(':') else {
        return false;
    };
    if comment_free[..colon].trim().is_empty() {
        return false;
    }
    is_block_indicator(comment_free[colon + 1..].trim())
}

fn is_block_indicator(value: &str) -> bool {
    let rest = match value.as_bytes().first() {
        Some(b'|' | b'>') => &value[1..],
        _ => return false,
    };
    let rest = rest
        .strip_prefix('+')
        .or_else(|| rest.strip_prefix('-'))
        .unwrap_or(rest);
    rest.trim_start_matches(|ch: char| ch.is_ascii_digit())
        .trim()
        .is_empty()
}

pub(crate) fn contains_mapping_key(line: &str, key: &str) -> bool {
    let comment_free = line.split('#').next().unwrap_or(line);
    let mut rest = comment_free;
    while let Some(idx) = rest.find(key) {
        let before_ok = idx == 0 || !is_key_char(rest.as_bytes()[idx - 1]);
        let after = &rest[idx + key.len()..];
        let after_ok = after.trim_start().starts_with(':')
            && after
                .chars()
                .next()
                .is_none_or(|ch| ch == ':' || ch.is_whitespace());
        if before_ok && after_ok {
            return true;
        }
        rest = &rest[idx + key.len()..];
    }
    false
}

fn is_key_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'
}
