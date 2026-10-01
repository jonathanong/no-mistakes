pub(super) fn split_body(definition: &str) -> (String, Option<String>) {
    if let Some((header, body)) = dollar_body(definition) {
        return (header, Some(body));
    }
    atomic_body(definition).unwrap_or_else(|| (definition.to_string(), None))
}

pub(super) fn skip_dollar_body(definition: &str, index: usize) -> Option<usize> {
    let (tag, open_end) = opening_dollar(definition, index)?;
    let close = format!("${tag}$");
    let relative = definition[open_end..].find(&close)?;
    Some(open_end + relative + close.len())
}

fn dollar_body(definition: &str) -> Option<(String, String)> {
    let mut index = 0;
    while index < definition.len() {
        if let Some(next) = skip_ignored(definition, index) {
            index = next;
            continue;
        }
        if is_word_at(definition, index, "as") {
            let mut cursor = index + 2;
            while definition
                .as_bytes()
                .get(cursor)
                .is_some_and(u8::is_ascii_whitespace)
            {
                cursor += 1;
            }
            if let Some((tag, open_end)) = opening_dollar(definition, cursor) {
                let close = format!("${tag}$");
                if let Some(relative) = definition[open_end..].find(&close) {
                    return Some((
                        definition[..index].to_string(),
                        definition[open_end..open_end + relative].to_string(),
                    ));
                }
            }
        }
        index += definition[index..].chars().next()?.len_utf8();
    }
    None
}

fn atomic_body(definition: &str) -> Option<(String, Option<String>)> {
    let mut index = 0;
    while index < definition.len() {
        if let Some(next) = skip_ignored(definition, index) {
            index = next;
            continue;
        }
        if is_word_at(definition, index, "begin") {
            let mut cursor = index + "begin".len();
            while definition
                .as_bytes()
                .get(cursor)
                .is_some_and(u8::is_ascii_whitespace)
            {
                cursor += 1;
            }
            if is_word_at(definition, cursor, "atomic") {
                let body_start = cursor + "atomic".len();
                let end = find_word(definition, body_start, "end")?;
                return Some((
                    definition[..index].to_string(),
                    Some(definition[body_start..end].to_string()),
                ));
            }
        }
        index += definition[index..].chars().next()?.len_utf8();
    }
    None
}

fn opening_dollar(definition: &str, index: usize) -> Option<(String, usize)> {
    let rest = definition[index..].strip_prefix('$')?;
    let end = rest.find('$')?;
    Some((rest[..end].to_string(), index + end + 2))
}

fn find_word(definition: &str, mut index: usize, word: &str) -> Option<usize> {
    while index < definition.len() {
        if let Some(next) = skip_ignored(definition, index) {
            index = next;
            continue;
        }
        if is_word_at(definition, index, word) {
            return Some(index);
        }
        index += definition[index..].chars().next()?.len_utf8();
    }
    None
}

fn skip_ignored(definition: &str, index: usize) -> Option<usize> {
    let rest = &definition[index..];
    if rest.starts_with("--") {
        return Some(index + rest.find('\n').unwrap_or(rest.len()));
    }
    if rest.starts_with("/*") {
        return Some(index + rest.find("*/").map(|end| end + 2).unwrap_or(rest.len()));
    }
    let quote = rest.chars().next()?;
    if quote == '\'' || quote == '"' {
        return Some(skip_quoted(definition, index, quote));
    }
    None
}

fn skip_quoted(definition: &str, index: usize, quote: char) -> usize {
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

fn is_word_at(text: &str, index: usize, word: &str) -> bool {
    let Some(slice) = text.get(index..index + word.len()) else {
        return false;
    };
    if !slice.eq_ignore_ascii_case(word) {
        return false;
    }
    let before_ok = index == 0 || !is_ident_byte(&text.as_bytes()[index - 1]);
    let after = text.as_bytes().get(index + word.len());
    before_ok && after.is_none_or(|byte| !is_ident_byte(byte))
}

fn is_ident_byte(byte: &u8) -> bool {
    byte.is_ascii_alphanumeric() || *byte == b'_'
}
