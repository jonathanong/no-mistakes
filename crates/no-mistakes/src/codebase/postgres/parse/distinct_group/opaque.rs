use super::word_boundary_before;

pub(super) fn skip_comment(sql: &str, index: usize) -> Option<usize> {
    match sql.as_bytes().get(index..index + 2)? {
        b"--" => Some(skip_line_comment(sql, index)),
        b"/*" => Some(skip_block_comment(sql, index)),
        _ => None,
    }
}

pub(super) fn skip_opaque(sql: &str, index: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    match bytes.get(index)? {
        b'-' if bytes.get(index + 1) == Some(&b'-') => Some(skip_line_comment(sql, index)),
        b'/' if bytes.get(index + 1) == Some(&b'*') => Some(skip_block_comment(sql, index)),
        b'\'' => Some(skip_quoted(sql, index, b'\'', is_escape_string(sql, index))),
        b'"' => Some(skip_quoted(sql, index, b'"', false)),
        // `foo$tag$` is an identifier, not a dollar-quote opener.
        b'$' if word_boundary_before(sql, index) => skip_dollar(sql, index),
        _ => None,
    }
}

fn skip_line_comment(sql: &str, index: usize) -> usize {
    sql[index..]
        .find('\n')
        .map(|offset| index + offset)
        .unwrap_or(sql.len())
}

/// PostgreSQL block comments nest.
fn skip_block_comment(sql: &str, index: usize) -> usize {
    let bytes = sql.as_bytes();
    let mut depth = 1;
    let mut at = index + 2;
    while at < bytes.len() {
        match bytes.get(at..at + 2) {
            Some(b"/*") => {
                depth += 1;
                at += 2;
            }
            Some(b"*/") => {
                depth -= 1;
                at += 2;
                if depth == 0 {
                    return at;
                }
            }
            _ => at += 1,
        }
    }
    sql.len()
}

/// `E'...'` strings treat a backslash as an escape, so `\'` does not close them.
fn is_escape_string(sql: &str, quote_at: usize) -> bool {
    quote_at > 0
        && matches!(sql.as_bytes()[quote_at - 1], b'e' | b'E')
        && word_boundary_before(sql, quote_at - 1)
}

fn skip_quoted(sql: &str, mut index: usize, quote: u8, backslash_escapes: bool) -> usize {
    let bytes = sql.as_bytes();
    index += 1;
    while index < bytes.len() {
        if backslash_escapes && bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        if bytes[index] == quote {
            if bytes.get(index + 1) == Some(&quote) {
                index += 2;
                continue;
            }
            return index + 1;
        }
        index += 1;
    }
    index
}

fn skip_dollar(sql: &str, start: usize) -> Option<usize> {
    let mut index = start + 1;
    while let Some(character) = sql[index..].chars().next() {
        if character.is_alphanumeric() || character == '_' {
            index += character.len_utf8();
        } else {
            break;
        }
    }
    if sql.as_bytes().get(index) != Some(&b'$') {
        return None;
    }
    let tag = &sql[start..=index];
    let rest = index + 1;
    Some(
        sql[rest..]
            .find(tag)
            .map(|offset| rest + offset + tag.len())
            .unwrap_or(sql.len()),
    )
}
