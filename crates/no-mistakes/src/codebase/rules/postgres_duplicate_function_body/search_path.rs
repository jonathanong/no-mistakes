pub(super) fn fold(clause: &str) -> String {
    let mut parts = Vec::new();
    let mut index = 0;
    let bytes = clause.as_bytes();
    while index < clause.len() {
        if clause[index..].starts_with("--") {
            index += clause[index..].find('\n').unwrap_or(clause.len() - index);
            continue;
        }
        if clause[index..].starts_with("/*") {
            index = skip_comment(clause, index);
            continue;
        }
        if bytes[index] == b'"' {
            let end = skip_quote(clause, index);
            parts.push(clause[index..end].to_string());
            index = end;
            continue;
        }
        if !is_name_byte(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        while index < clause.len() && is_name_byte(bytes[index]) {
            index += 1;
        }
        parts.push(clause[start..index].to_ascii_lowercase());
    }
    parts.join(" ")
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn skip_quote(clause: &str, index: usize) -> usize {
    let mut cursor = index + 1;
    let bytes = clause.as_bytes();
    while cursor < clause.len() {
        if bytes[cursor] == b'"' {
            if bytes.get(cursor + 1) == Some(&b'"') {
                cursor += 2;
                continue;
            }
            return cursor + 1;
        }
        cursor += 1;
    }
    clause.len()
}

fn skip_comment(clause: &str, index: usize) -> usize {
    let mut depth = 1i32;
    let mut cursor = index + 2;
    while cursor < clause.len() && depth > 0 {
        if clause[cursor..].starts_with("/*") {
            depth += 1;
            cursor += 2;
            continue;
        }
        if clause[cursor..].starts_with("*/") {
            depth -= 1;
            cursor += 2;
            continue;
        }
        cursor += clause[cursor..].chars().next().map_or(1, char::len_utf8);
    }
    cursor
}
