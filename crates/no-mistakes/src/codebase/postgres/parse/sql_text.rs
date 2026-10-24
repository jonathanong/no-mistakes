/// Top-level SQL statements with the line of their first code token.
///
/// Semicolons inside quotes, dollar quotes, and comments stay in the statement.
/// A leading `--` comment is not the statement line, so a next-line suppression
/// still points at the SQL.
fn split_sql(sql: &str, data: &mut Vec<(usize, usize, usize)>) -> Vec<(usize, usize, String)> {
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
            code_line = Some((line, index));
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
            b'$' => index = dollar::skip(sql, index, &mut line),
            b';' => {
                push(
                    &mut out,
                    sql,
                    start,
                    index,
                    code_line.unwrap_or((start_line, start)),
                );
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
        code_line.unwrap_or((start_line, start)),
    );
    out
}

fn push(
    out: &mut Vec<(usize, usize, String)>,
    sql: &str,
    start: usize,
    end: usize,
    at: (usize, usize),
) {
    let chunk = sql.get(start..end).unwrap_or_default();
    if chunk.chars().any(|ch| !ch.is_whitespace()) {
        out.push((at.0, at.1, chunk.to_string()));
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

/// COPY payloads are data, so tokenizer quotes and semicolons must not own them.
pub(crate) fn normalize_copy_data(sql: &str) -> std::borrow::Cow<'_, str> {
    normalize_copy(sql, true)
}

/// Token-only consumers need SQL positions without COPY payload or terminator tokens.
pub(crate) fn normalize_copy_data_for_tokens(sql: &str) -> std::borrow::Cow<'_, str> {
    normalize_copy(sql, false)
}

fn normalize_copy(sql: &str, retain_terminator: bool) -> std::borrow::Cow<'_, str> {
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
        let blank_end = if retain_terminator { end } else { after };
        for byte in &mut bytes[start..blank_end] {
            if !matches!(*byte, b'\n' | b'\r') {
                *byte = b' ';
            }
        }
        if retain_terminator && after > end {
            // sqlparser needs a delimiter after its COPY data terminator.
            let terminator_end = end + sql[end..after].find("\\.").unwrap() + 2;
            bytes.insert(terminator_end, b';');
        }
    }
    std::borrow::Cow::Owned(String::from_utf8(bytes).unwrap())
}

mod dollar;

#[cfg(test)]
mod tests;

/// First-code columns are projected from the same split, in one forward character pass.
pub(super) fn located_statements(sql: &str) -> Vec<(usize, usize, String)> {
    let (mut previous, mut column) = (0, 1);
    split_sql(sql, &mut Vec::new())
        .into_iter()
        .map(|(line, code, text)| {
            for character in sql[previous..code].chars() {
                column = if character == '\n' { 1 } else { column + 1 };
            }
            previous = code;
            (line, column, text)
        })
        .collect()
}
