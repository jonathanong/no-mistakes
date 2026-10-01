/// Top-level SQL statements with the line of their first code token.
///
/// Semicolons inside quotes, dollar quotes, and comments stay in the statement.
/// A leading `--` comment is not the statement line, so a next-line suppression
/// still points at the SQL.
pub(super) fn top_level_statements(sql: &str) -> Vec<(usize, String)> {
    let bytes = sql.as_bytes();
    let mut index = 0usize;
    let mut line = 1usize;
    let mut start = 0usize;
    let mut start_line = 1usize;
    let mut code_line = None;
    let mut out = Vec::new();
    while index < bytes.len() {
        if code_line.is_none()
            && !bytes[index].is_ascii_whitespace()
            && !starts_comment(bytes, index)
        {
            code_line = Some(line);
        }
        match bytes[index] {
            b'\n' => {
                line += 1;
                index += 1;
            }
            b'-' if bytes.get(index + 1) == Some(&b'-') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                {
                    if bytes[index] == b'\n' {
                        line += 1;
                    }
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
            }
            quote @ (b'\'' | b'"') => index = skip_quoted(bytes, index, quote, &mut line),
            b'$' => index = skip_dollar(bytes, index, &mut line),
            b';' => {
                push(&mut out, sql, start, index, code_line.unwrap_or(start_line));
                index += 1;
                start = index;
                start_line = line;
                code_line = None;
            }
            _ => index += 1,
        }
    }
    push(
        &mut out,
        sql,
        start,
        sql.len(),
        code_line.unwrap_or(start_line),
    );
    out
}

fn push(out: &mut Vec<(usize, String)>, sql: &str, start: usize, end: usize, line: usize) {
    let chunk = sql.get(start..end).unwrap_or_default();
    if chunk.chars().any(|ch| !ch.is_whitespace()) {
        out.push((line, chunk.to_string()));
    }
}

fn starts_comment(bytes: &[u8], index: usize) -> bool {
    matches!(
        (bytes.get(index), bytes.get(index + 1)),
        (Some(b'-'), Some(b'-')) | (Some(b'/'), Some(b'*'))
    )
}

fn skip_quoted(bytes: &[u8], mut index: usize, quote: u8, line: &mut usize) -> usize {
    index += 1;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            *line += 1;
        }
        if bytes[index] == quote {
            return index + 1;
        }
        index += 1;
    }
    index
}

fn skip_dollar(bytes: &[u8], index: usize, line: &mut usize) -> usize {
    let mut tag_end = index + 1;
    while tag_end < bytes.len()
        && bytes[tag_end] != b'$'
        && (bytes[tag_end].is_ascii_alphanumeric() || bytes[tag_end] == b'_')
    {
        tag_end += 1;
    }
    if tag_end >= bytes.len() || bytes[tag_end] != b'$' {
        return index + 1;
    }
    let tag = &bytes[index..=tag_end];
    let mut cursor = index + tag.len();
    while cursor + tag.len() <= bytes.len() {
        if bytes[cursor] == b'\n' {
            *line += 1;
        }
        if bytes[cursor..].starts_with(tag) {
            return cursor + tag.len();
        }
        cursor += 1;
    }
    bytes.len()
}
