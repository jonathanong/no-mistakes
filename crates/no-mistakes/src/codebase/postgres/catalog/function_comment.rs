pub(super) fn skip_quoted(definition: &str, index: usize, quote: char) -> usize {
    let mut chars = definition[index + quote.len_utf8()..].char_indices();
    while let Some((offset, character)) = chars.next() {
        if character == quote {
            let next = chars.clone().next();
            if next.is_some_and(|(_, doubled)| doubled == quote) {
                chars.next();
                continue;
            }
            return index + quote.len_utf8() + offset + quote.len_utf8();
        }
    }
    definition.len()
}

pub(super) fn skip_block_comment(definition: &str, index: usize) -> usize {
    let mut depth = 1i32;
    let mut cursor = index + 2;
    while cursor < definition.len() && depth > 0 {
        if definition[cursor..].starts_with("/*") {
            depth += 1;
            cursor += 2;
            continue;
        }
        if definition[cursor..].starts_with("*/") {
            depth -= 1;
            cursor += 2;
            continue;
        }
        let step = definition[cursor..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
        cursor += step;
    }
    cursor
}

pub(super) fn skip_escape_string(definition: &str, index: usize) -> Option<usize> {
    let before = definition[..index].chars().next_back()?;
    if !before.eq_ignore_ascii_case(&'e') {
        return None;
    }
    let prefix = index - before.len_utf8();
    if prefix > 0 {
        let earlier = definition[..prefix].chars().next_back()?;
        if earlier.is_alphanumeric() || earlier == '_' {
            return None;
        }
    }
    let mut cursor = index + 1;
    while cursor < definition.len() {
        let character = definition[cursor..].chars().next()?;
        let len = character.len_utf8();
        if character == '\\' {
            cursor += len;
            if let Some(escaped) = definition[cursor..].chars().next() {
                cursor += escaped.len_utf8();
            }
            continue;
        }
        if character == '\'' {
            if definition[cursor + len..].starts_with('\'') {
                cursor += len + 1;
                continue;
            }
            return Some(cursor + len);
        }
        cursor += len;
    }
    Some(definition.len())
}
