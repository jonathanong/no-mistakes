use super::model::CatalogFunction;

pub(super) fn function_from_definition(key: &str, definition: &str) -> CatalogFunction {
    let (name, signature) = split_key(key);
    let (header, body) = split_body(definition);
    CatalogFunction {
        key: key.to_string(),
        name,
        signature,
        language: language(definition),
        returns_trigger: returns_trigger(&header),
        definition: definition.to_string(),
        body,
    }
}

fn split_key(key: &str) -> (String, Option<String>) {
    match key.split_once('(') {
        None => (key.trim().to_string(), None),
        Some((name, rest)) => {
            let signature = rest.trim().strip_suffix(')').unwrap_or(rest.trim());
            (name.trim().to_string(), Some(signature.trim().to_string()))
        }
    }
}

fn split_body(definition: &str) -> (String, Option<String>) {
    let bytes = definition.as_bytes();
    for (index, _) in definition.char_indices() {
        if !is_word_at(definition, index, "as") {
            continue;
        }
        let mut cursor = index + 2;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if let Some((tag, open_end)) = opening_dollar(definition, cursor) {
            let close = format!("${tag}$");
            if let Some(relative) = definition[open_end..].find(&close) {
                return (
                    definition[..index].to_string(),
                    Some(definition[open_end..open_end + relative].to_string()),
                );
            }
        }
    }
    (definition.to_string(), None)
}

fn opening_dollar(definition: &str, index: usize) -> Option<(String, usize)> {
    let rest = definition[index..].strip_prefix('$')?;
    let end = rest.find('$')?;
    Some((rest[..end].to_string(), index + end + 2))
}

fn language(definition: &str) -> Option<String> {
    let bytes = definition.as_bytes();
    for (index, _) in definition.char_indices() {
        if !is_word_at(definition, index, "language") {
            continue;
        }
        let mut cursor = index + "language".len();
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let start = cursor;
        while bytes.get(cursor).is_some_and(is_ident_byte) {
            cursor += 1;
        }
        if cursor > start {
            return Some(definition[start..cursor].to_ascii_lowercase());
        }
        return None;
    }
    None
}

fn returns_trigger(header: &str) -> bool {
    let lower = header.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(index) = rest.find("returns") {
        let before = index == 0 || !is_ident_byte(&rest.as_bytes()[index - 1]);
        let after = rest[index + "returns".len()..].trim_start();
        if before && word_starts(after, "trigger") {
            return true;
        }
        rest = &rest[index + "returns".len()..];
    }
    false
}

fn word_starts(text: &str, word: &str) -> bool {
    let Some(rest) = text.get(..word.len()) else {
        return false;
    };
    if !rest.eq_ignore_ascii_case(word) {
        return false;
    }
    text.as_bytes()
        .get(word.len())
        .is_none_or(|byte| !is_ident_byte(byte))
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
