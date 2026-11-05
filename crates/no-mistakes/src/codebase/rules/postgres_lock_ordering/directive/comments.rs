pub(crate) fn comment_contains_directive(text: &str, directive: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if let Some(end) = quoted_end(bytes, index) {
            index = end;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let start = index + 2;
            let end = block_end(bytes, start);
            if text[start..end].contains(directive) {
                return true;
            }
            index = (end + 2).min(bytes.len());
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"--") {
            let start = index + 2;
            let end = text[start..]
                .find('\n')
                .map_or(bytes.len(), |relative| start + relative);
            if text[start..end].contains(directive) {
                return true;
            }
            index = end;
            continue;
        }
        index += 1;
    }
    false
}

fn quoted_end(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    if matches!(quote, b'\'' | b'"' | b'`') {
        let escape = quote == b'`'
            || quote == b'\''
                && start > 0
                && matches!(bytes[start - 1], b'e' | b'E')
                && (start == 1 || !sql_identifier_byte(bytes[start - 2]));
        let mut index = start + 1;
        while index < bytes.len() {
            if escape && bytes[index] == b'\\' {
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
        return Some(index);
    }
    if quote != b'$' {
        return None;
    }
    let mut end = start + 1;
    if bytes.get(end).is_some_and(u8::is_ascii_digit) {
        return None;
    }
    while bytes
        .get(end)
        .is_some_and(|byte| sql_identifier_byte(*byte))
    {
        end += 1;
    }
    if bytes.get(end) != Some(&b'$') {
        return None;
    }
    let delimiter = &bytes[start..=end];
    Some(
        bytes[end + 1..]
            .windows(delimiter.len())
            .position(|window| window == delimiter)
            .map_or(bytes.len(), |relative| end + 1 + relative + delimiter.len()),
    )
}

fn sql_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80
}

fn block_end(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    let mut depth = 1;
    while index + 1 < bytes.len() {
        if bytes.get(index..index + 2) == Some(b"/*") {
            depth += 1;
            index += 2;
        } else if bytes.get(index..index + 2) == Some(b"*/") {
            depth -= 1;
            if depth == 0 {
                return index;
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    bytes.len()
}
