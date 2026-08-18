pub(super) const DEFAULT_SAFE_DIRECTIVE: &str = "deadlock-safe";
const DIRECTIVE_LOOKBACK: usize = 200;

pub(super) fn contains_for_update(sql: &str) -> bool {
    sql.to_ascii_lowercase().contains("for update")
}

pub(super) fn has_safe_directive(source: &str, line: u32, sql: &str, directive: &str) -> bool {
    if directive.is_empty() {
        return false;
    }
    comment_contains_directive(&lookback_window(source, line), directive)
        || comment_contains_directive(sql, directive)
}

fn lookback_window(source: &str, line: u32) -> String {
    let offset = call_offset(source, line);
    let start = floor_char_boundary(source, offset.saturating_sub(DIRECTIVE_LOOKBACK));
    let end = floor_char_boundary(source, offset);
    source.get(start..end).unwrap_or("").to_string()
}

pub(super) fn call_offset(source: &str, line: u32) -> usize {
    let line_start = line_start_offset(source, line);
    let haystack = source.get(line_start..).unwrap_or("");
    let rel = haystack
        .to_ascii_lowercase()
        .find("for update")
        .unwrap_or(0);
    line_start + rel
}

pub(super) fn line_start_offset(source: &str, line: u32) -> usize {
    if line <= 1 {
        return 0;
    }
    source
        .match_indices('\n')
        .nth(line.saturating_sub(2) as usize)
        .map(|(idx, _)| idx + 1)
        .unwrap_or(source.len())
}

pub(super) fn floor_char_boundary(source: &str, index: usize) -> usize {
    if index >= source.len() {
        return source.len();
    }
    if source.is_char_boundary(index) {
        return index;
    }
    (0..index)
        .rev()
        .find(|idx| source.is_char_boundary(*idx))
        .unwrap_or(0)
}

pub(super) fn comment_contains_directive(text: &str, directive: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            let start = index + 2;
            let rest = &text[start..];
            if let Some(end) = rest.find("*/") {
                if rest[..end].contains(directive) {
                    return true;
                }
                index = start + end + 2;
                continue;
            }
            return rest.contains(directive);
        }
        if bytes[index] == b'-' && bytes.get(index + 1) == Some(&b'-') {
            let start = index + 2;
            let rest = &text[start..];
            let end = rest.find('\n').unwrap_or(rest.len());
            if rest[..end].contains(directive) {
                return true;
            }
            index = start + end;
            continue;
        }
        index += 1;
    }
    false
}
