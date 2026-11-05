pub(crate) const DEFAULT_SAFE_DIRECTIVE: &str = "deadlock-safe";
const DIRECTIVE_LOOKBACK: usize = 200;

pub(super) fn contains_for_update(sql: &str) -> bool {
    lock_clause_offset(&sql.to_ascii_lowercase()).is_some()
}

/// Offset of the first exclusive-lock clause in already-lowercased text.
pub(super) fn lock_clause_offset(lowered: &str) -> Option<usize> {
    let bytes = lowered.as_bytes();
    let mut index = 0;
    while index + 3 <= bytes.len() {
        if keyword_at(bytes, index, b"for") {
            let (next, spaced) = skip_sql_trivia(bytes, index + 3);
            if spaced && keyword_at(bytes, next, b"update") {
                return Some(index);
            }
            if spaced && keyword_at(bytes, next, b"no") {
                let (key, spaced) = skip_sql_trivia(bytes, next + 2);
                if spaced && keyword_at(bytes, key, b"key") {
                    let (update, spaced) = skip_sql_trivia(bytes, key + 3);
                    if spaced && keyword_at(bytes, update, b"update") {
                        return Some(index);
                    }
                }
            }
        }
        index += 1;
    }
    None
}

fn keyword_at(bytes: &[u8], start: usize, word: &[u8]) -> bool {
    let end = start + word.len();
    bytes.get(start..end) == Some(word)
        && (start == 0 || !identifier_byte(bytes[start - 1]))
        && (end == bytes.len() || !identifier_byte(bytes[end]))
}

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn skip_sql_trivia(bytes: &[u8], mut index: usize) -> (usize, bool) {
    let start = index;
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            index += 2;
            while index + 1 < bytes.len() && bytes.get(index..index + 2) != Some(b"*/") {
                index += 1;
            }
            if bytes.get(index..index + 2) != Some(b"*/") {
                return (bytes.len(), true);
            }
            index += 2;
        } else if bytes.get(index..index + 2) == Some(b"--") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else {
            return (index, index > start);
        }
    }
}

pub(crate) fn has_safe_directive(source: &str, line: u32, sql: &str, directive: &str) -> bool {
    if directive.is_empty() {
        return false;
    }
    let call_start = call_offset(source, line);
    let start = floor_char_boundary(source, call_start.saturating_sub(DIRECTIVE_LOOKBACK));
    let end = floor_char_boundary(source, call_start);
    comment_contains_directive(&source[start..end], directive)
        || comment_contains_directive(sql, directive)
}

pub(super) fn has_safe_variant_directive(
    source: &str,
    call_start: u32,
    comment_spans: &[(u32, u32)],
    sql: &str,
    directive: &str,
) -> bool {
    if directive.is_empty() {
        return false;
    }
    let start = call_start.saturating_sub(DIRECTIVE_LOOKBACK as u32);
    comment_spans.iter().any(|&(comment_start, comment_end)| {
        comment_start >= start
            && comment_end <= call_start
            && comment_contains_directive(
                &source[comment_start as usize..comment_end as usize],
                directive,
            )
    }) || comment_contains_directive(sql, directive)
}

pub(super) fn call_offset(source: &str, line: u32) -> usize {
    let line_start = line_start_offset(source, line);
    let haystack = &source[line_start..];
    let rel = lock_clause_offset(&haystack.to_ascii_lowercase()).unwrap_or(0);
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
        if matches!(bytes[index], b'\'' | b'"' | b'`') {
            let quote = bytes[index];
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == quote {
                    index += 1;
                    if bytes.get(index) == Some(&quote) && quote != b'`' {
                        index += 1;
                    } else {
                        break;
                    }
                } else {
                    index += 1;
                }
            }
            continue;
        }
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
