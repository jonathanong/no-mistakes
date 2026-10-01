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
        if !name_char_at(clause, index) {
            index += clause[index..].chars().next().map_or(1, char::len_utf8);
            continue;
        }
        let start = index;
        while index < clause.len() && name_char_at(clause, index) {
            index += clause[index..].chars().next().map_or(1, char::len_utf8);
        }
        parts.push(clause[start..index].to_ascii_lowercase());
    }
    parts.join(" ")
}

fn skip_dollar(header: &str, index: usize) -> Option<usize> {
    let rest = header[index..].strip_prefix('$')?;
    let end = rest.find('$')?;
    let tag = &rest[..end];
    let valid = tag.is_empty()
        || (tag
            .chars()
            .next()
            .is_some_and(|character| character.is_alphabetic() || character == '_')
            && tag
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_'));
    if !valid {
        return None;
    }
    let body_start = index + end + 2;
    let close = format!("${tag}$");
    let relative = header[body_start..].find(&close)?;
    Some(body_start + relative + close.len())
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_ident_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '$'
}

fn name_char_at(text: &str, index: usize) -> bool {
    text[index..].chars().next().is_some_and(is_ident_char)
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
    let before = index == 0 || !text[..index].chars().next_back().is_some_and(is_ident_char);
    let after = text
        .get(index + word.len()..)
        .and_then(|rest| rest.chars().next());
    before && after.is_none_or(|character| !is_ident_char(character))
}

fn skip_token(header: &str, index: usize) -> usize {
    let bytes = header.as_bytes();
    if bytes[index] == b'$' {
        if let Some(end) = skip_dollar(header, index) {
            return end;
        }
    }
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

fn ends_clause(header: &str, index: usize) -> bool {
    "language as begin set immutable stable volatile"
        .split(' ')
        .any(|word| word_at(header, index, word))
}

fn clause_end(header: &str, mut index: usize) -> usize {
    let mut saw_value = false;
    while index < header.len() {
        if header.as_bytes()[index] == b';' {
            return index;
        }
        let next = skip_gap(header, index);
        if next != index {
            index = next;
            continue;
        }
        if saw_value && ends_clause(header, index) {
            return index;
        }
        let marker = header.as_bytes()[index] == b'='
            || word_at(header, index, "to")
            || word_at(header, index, "from");
        if !marker {
            saw_value = true;
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
