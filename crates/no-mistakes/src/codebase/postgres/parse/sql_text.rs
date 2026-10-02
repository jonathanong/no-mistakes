/// Top-level SQL statements with the line of their first code token.
///
/// Semicolons inside quotes, dollar quotes, and comments stay in the statement.
/// A leading `--` comment is not the statement line, so a next-line suppression
/// still points at the SQL.
fn split_sql(sql: &str, data: &mut Vec<(usize, usize, usize)>) -> Vec<(usize, String)> {
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
                let mut depth = 1usize;
                while index < bytes.len() && depth > 0 {
                    if bytes[index] == b'\n' {
                        line += 1;
                    }
                    if bytes[index..].starts_with(b"/*") {
                        depth += 1;
                        index += 2;
                        continue;
                    }
                    if bytes[index..].starts_with(b"*/") {
                        depth -= 1;
                        index += 2;
                        continue;
                    }
                    index += 1;
                }
            }
            quote @ (b'\'' | b'"') => index = skip_quoted(bytes, index, quote, &mut line),
            b'$' => index = skip_dollar(bytes, index, &mut line),
            b';' => {
                push(&mut out, sql, start, index, code_line.unwrap_or(start_line));
                let copy = super::copy_data::is_copy_stdin(&sql[start..index]);
                index += 1;
                if copy {
                    let (payload_end, after) = super::copy_data::payload_end(sql, index);
                    data.push((index, payload_end, after));
                    line += sql[index..after]
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count();
                    index = after;
                }
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
    let escaped = quote == b'\''
        && index > 0
        && matches!(bytes[index - 1], b'e' | b'E')
        && (index < 2 || !(bytes[index - 2].is_ascii_alphanumeric() || bytes[index - 2] == b'_'));
    index += 1;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            *line += 1;
        }
        if escaped && bytes[index] == b'\\' {
            if bytes.get(index + 1) == Some(&b'\n') {
                *line += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        if bytes[index] == quote && bytes.get(index + 1) == Some(&quote) {
            index += 2;
            continue;
        }
        if bytes[index] == quote {
            return index + 1;
        }
        index += 1;
    }
    index
}

fn skip_dollar(bytes: &[u8], index: usize, line: &mut usize) -> usize {
    // Dollar signs are legal inside unquoted PostgreSQL identifiers.
    if index > 0
        && (bytes[index - 1].is_ascii_alphanumeric()
            || matches!(bytes[index - 1], b'_' | b'$')
            || !bytes[index - 1].is_ascii())
    {
        return index + 1;
    }
    if bytes.get(index + 1).is_some_and(u8::is_ascii_digit) {
        return index + 1;
    }
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

/// COPY payloads are data, so tokenizer quotes and semicolons must not own them.
pub(crate) fn normalize_copy_data(sql: &str) -> std::borrow::Cow<'_, str> {
    if !sql
        .as_bytes()
        .windows(5)
        .any(|word| word.eq_ignore_ascii_case(b"STDIN"))
    {
        return std::borrow::Cow::Borrowed(sql);
    }
    let mut ranges = Vec::new();
    split_sql(sql, &mut ranges);
    if ranges.is_empty() {
        return std::borrow::Cow::Borrowed(sql);
    }
    let mut bytes = sql.as_bytes().to_vec();
    for (start, end, after) in ranges.into_iter().rev() {
        for byte in &mut bytes[start..end] {
            if !matches!(*byte, b'\n' | b'\r') {
                *byte = b' ';
            }
        }
        if after > end {
            // sqlparser needs a delimiter after its COPY data terminator.
            let terminator_end = end + sql[end..after].find("\\.").unwrap() + 2;
            bytes.insert(terminator_end, b';');
        }
    }
    std::borrow::Cow::Owned(String::from_utf8(bytes).unwrap())
}
