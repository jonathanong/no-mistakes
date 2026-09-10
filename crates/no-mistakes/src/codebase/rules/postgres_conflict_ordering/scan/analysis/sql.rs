use std::ops::Range;

pub(in super::super) fn contains_insert_conflict(sql: &str) -> bool {
    contains_keyword(sql, "insert")
        && contains_keyword(sql, "on")
        && contains_keyword(sql, "conflict")
}

#[cfg(test)]
pub(in super::super) fn contains_insert(sql: &str) -> bool {
    contains_keyword(sql, "insert")
}

/// Split SQL on statement terminators outside quoted values and comments.
/// Each returned line points to the statement's first code token.
pub(in super::super) fn sql_statements(sql: &str) -> Vec<(u32, &str)> {
    let mut statements = Vec::new();
    let mut start = 0;
    for span in sql_code_spans(sql) {
        for (relative, byte) in sql.as_bytes()[span.clone()].iter().enumerate() {
            if *byte != b';' {
                continue;
            }
            let end = span.start + relative + 1;
            let statement = &sql[start..end];
            statements.push((statement_line(sql, start, statement), statement));
            start = end;
        }
    }
    if start < sql.len() {
        let statement = &sql[start..];
        statements.push((statement_line(sql, start, statement), statement));
    }
    statements
}

fn contains_keyword(sql: &str, keyword: &str) -> bool {
    let bytes = sql.as_bytes();
    sql_code_spans(sql).into_iter().any(|span| {
        let mut index = span.start;
        while index < span.end {
            if !is_identifier_byte(bytes[index]) {
                index += 1;
                continue;
            }
            let start = index;
            while index < span.end && is_identifier_byte(bytes[index]) {
                index += 1;
            }
            if bytes[start..index].eq_ignore_ascii_case(keyword.as_bytes()) {
                return true;
            }
        }
        false
    })
}

fn statement_line(sql: &str, start: usize, statement: &str) -> u32 {
    let offset = sql_code_spans(statement)
        .into_iter()
        .flat_map(|span| span.filter(|index| !statement.as_bytes()[*index].is_ascii_whitespace()))
        .next()
        .unwrap_or(0);
    sql.as_bytes()[..start + offset]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count() as u32
        + 1
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn sql_code_spans(sql: &str) -> Vec<Range<usize>> {
    let bytes = sql.as_bytes();
    let mut spans = Vec::new();
    let mut code_start = 0;
    let mut index = 0;
    while index < bytes.len() {
        let quoted_end = match bytes[index] {
            b'\'' => Some(quoted_end(bytes, index, b'\'')),
            b'"' => Some(quoted_end(bytes, index, b'"')),
            b'-' if bytes.get(index + 1) == Some(&b'-') => Some(line_comment_end(bytes, index)),
            b'/' if bytes.get(index + 1) == Some(&b'*') => Some(block_comment_end(bytes, index)),
            b'$' => dollar_quote_end(bytes, index),
            _ => None,
        };
        let Some(end) = quoted_end else {
            index += 1;
            continue;
        };
        if code_start < index {
            spans.push(code_start..index);
        }
        index = end;
        code_start = end;
    }
    if code_start < bytes.len() {
        spans.push(code_start..bytes.len());
    }
    spans
}

fn quoted_end(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == quote {
            if bytes.get(index + 1) == Some(&quote) {
                index += 2;
                continue;
            }
            return index + 1;
        }
        if quote == b'\'' && bytes[index] == b'\\' && index + 1 < bytes.len() {
            index += 2;
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn line_comment_end(bytes: &[u8], start: usize) -> usize {
    bytes[start + 2..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |relative| start + 2 + relative)
}

fn block_comment_end(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1;
    let mut index = start + 2;
    while index + 1 < bytes.len() {
        match (bytes[index], bytes[index + 1]) {
            (b'/', b'*') => {
                depth += 1;
                index += 2;
            }
            (b'*', b'/') => {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    return index;
                }
            }
            _ => index += 1,
        }
    }
    bytes.len()
}

fn dollar_quote_end(bytes: &[u8], start: usize) -> Option<usize> {
    let first = *bytes.get(start + 1)?;
    if !(first.is_ascii_alphabetic() || first == b'_') && first != b'$' {
        return None;
    }
    let mut delimiter_end = start + 2;
    if first != b'$' {
        while delimiter_end < bytes.len() && is_identifier_byte(bytes[delimiter_end]) {
            delimiter_end += 1;
        }
        if bytes.get(delimiter_end) != Some(&b'$') {
            return None;
        }
        delimiter_end += 1;
    }
    let delimiter = &bytes[start..delimiter_end];
    let mut index = delimiter_end;
    while index + delimiter.len() <= bytes.len() {
        if bytes[index..index + delimiter.len()] == *delimiter {
            return Some(index + delimiter.len());
        }
        index += 1;
    }
    Some(bytes.len())
}
