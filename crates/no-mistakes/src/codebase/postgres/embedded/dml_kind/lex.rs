pub(super) fn skip_trivia(sql: &str, mut index: usize) -> usize {
    let bytes = sql.as_bytes();
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        let skipped = match bytes.get(index).copied() {
            Some(b'-') if bytes.get(index + 1) == Some(&b'-') => {
                Some(line_comment_end(bytes, index))
            }
            Some(b'/') if bytes.get(index + 1) == Some(&b'*') => {
                Some(block_comment_end(bytes, index))
            }
            _ => None,
        };
        match skipped {
            Some(end) if end > index => index = end,
            _ => return index,
        }
    }
}

pub(super) fn skip_ident(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    if bytes.get(start) == Some(&b'"') {
        let end = sql[start + 1..].find('"')?;
        return Some(start + end + 2);
    }
    let end = bytes[start..]
        .iter()
        .position(|byte| !is_ident_byte(*byte))
        .map_or(bytes.len(), |relative| start + relative);
    (end > start).then_some(end)
}

pub(super) fn skip_balanced_paren(sql: &str, open: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        if let Some(end) = skip_delimited(sql, index) {
            index = end;
            continue;
        }
        match bytes[index] {
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub(super) fn starts_keyword(sql: &str, start: usize, keyword: &str) -> bool {
    let end = start.saturating_add(keyword.len());
    sql.get(start..end)
        .is_some_and(|text| text.eq_ignore_ascii_case(keyword))
        && !sql
            .as_bytes()
            .get(start.wrapping_sub(1))
            .copied()
            .is_some_and(is_ident_byte)
        && !sql.as_bytes().get(end).copied().is_some_and(is_ident_byte)
}

fn skip_delimited(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    match bytes.get(start).copied()? {
        b'\'' => Some(quoted_end(bytes, start, b'\'')),
        b'"' => Some(quoted_end(bytes, start, b'"')),
        b'-' if bytes.get(start + 1) == Some(&b'-') => Some(line_comment_end(bytes, start)),
        b'/' if bytes.get(start + 1) == Some(&b'*') => Some(block_comment_end(bytes, start)),
        b'$' => dollar_quote_end(bytes, start),
        _ => None,
    }
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
        .map_or(bytes.len(), |relative| start + 2 + relative + 1)
}

fn block_comment_end(bytes: &[u8], start: usize) -> usize {
    bytes[start + 2..]
        .windows(2)
        .position(|pair| pair == b"*/")
        .map_or(bytes.len(), |relative| start + relative + 4)
}

fn dollar_quote_end(bytes: &[u8], start: usize) -> Option<usize> {
    let first = *bytes.get(start + 1)?;
    if !(first.is_ascii_alphabetic() || first == b'_') && first != b'$' {
        return None;
    }
    let mut delimiter_end = start + 2;
    if first != b'$' {
        while delimiter_end < bytes.len() && is_ident_byte(bytes[delimiter_end]) {
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

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
