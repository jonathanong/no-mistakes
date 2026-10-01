pub(super) fn extract(header: &str) -> String {
    let mut index = 0;
    while index < header.len() {
        let next = skip_gap(header, index);
        if next != index {
            index = next;
            continue;
        }
        if word_at(header, index, "set") {
            let after = skip_gap(header, index + 3);
            if word_at(header, after, "search_path") {
                let start = after + "search_path".len();
                return fold(&header[start..clause_end(header, start)]);
            }
        }
        index = skip_token(header, index);
    }
    String::new()
}

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

fn skip_gap(header: &str, mut index: usize) -> usize {
    while index < header.len() {
        if header[index..].starts_with("--") {
            index += header[index..].find('\n').unwrap_or(header.len() - index);
            continue;
        }
        if header[index..].starts_with("/*") {
            index = skip_comment(header, index);
            continue;
        }
        if header.as_bytes()[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        break;
    }
    index
}

fn word_at(text: &str, index: usize, word: &str) -> bool {
    let Some(slice) = text.get(index..index + word.len()) else {
        return false;
    };
    if !slice.eq_ignore_ascii_case(word) {
        return false;
    }
    let before = index == 0 || !is_name_byte(text.as_bytes()[index - 1]);
    let after = text.as_bytes().get(index + word.len());
    before && after.is_none_or(|byte| !is_name_byte(*byte))
}

fn skip_token(header: &str, index: usize) -> usize {
    let bytes = header.as_bytes();
    if bytes[index] == b'\'' || bytes[index] == b'"' {
        return skip_quote(header, index);
    }
    if is_name_byte(bytes[index]) {
        let mut end = index + 1;
        while end < header.len() && is_name_byte(bytes[end]) {
            end += 1;
        }
        return end;
    }
    index + header[index..].chars().next().map_or(1, char::len_utf8)
}

fn clause_end(header: &str, mut index: usize) -> usize {
    while index < header.len() {
        if header.as_bytes()[index] == b';' {
            return index;
        }
        let next = skip_gap(header, index);
        if next != index {
            index = next;
            continue;
        }
        if word_at(header, index, "language")
            || word_at(header, index, "as")
            || word_at(header, index, "begin")
            || word_at(header, index, "set")
        {
            return index;
        }
        let skipped = skip_token(header, index);
        if skipped == index {
            return index;
        }
        index = skipped;
    }
    header.len()
}

fn skip_quote(clause: &str, index: usize) -> usize {
    let quote = clause.as_bytes()[index];
    let mut cursor = index + 1;
    let bytes = clause.as_bytes();
    while cursor < clause.len() {
        if bytes[cursor] == quote {
            if bytes.get(cursor + 1) == Some(&quote) {
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
