use super::model::CatalogFunction;

pub(super) fn function_from_definition(key: &str, definition: &str) -> CatalogFunction {
    let (name, signature) = split_key(key);
    let (header, body, body_span) = super::function_body::split_body(definition);
    let plain = strip_quotes_and_comments(&header);
    let modes = super::function_clauses::header_modes(definition, body_span);
    CatalogFunction {
        key: key.to_string(),
        name,
        signature,
        language: language(definition, body_span),
        returns_trigger: returns_clause(&plain, "trigger"),
        returns_event_trigger: returns_clause(&plain, "event_trigger"),
        definition: definition.to_string(),
        body,
        null_input: modes.null_input,
        security: modes.security,
        parallel: modes.parallel,
        leakproof: modes.leakproof,
        volatility: modes.volatility,
        return_contract: modes.return_contract,
        planner: modes.planner,
        body_span,
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

fn language(definition: &str, span: Option<(usize, usize)>) -> Option<String> {
    let bytes = definition.as_bytes();
    let mut index = 0;
    let mut depth = 0i32;
    while index < definition.len() {
        if let Some((start, end)) = span {
            if index >= start && index < end {
                index = end;
                continue;
            }
        }
        if let Some(end) = super::function_body::skip_noise(definition, index) {
            index = end;
            continue;
        }
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        if depth == 0 && is_word_at(definition, index, "language") {
            let mut cursor = index + "language".len();
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
            if let Some(name) = super::function_body::language_name(definition, cursor) {
                return Some(name);
            }
        }
        index += definition[index..].chars().next()?.len_utf8();
    }
    None
}

fn strip_quotes_and_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        if let Some(end) = super::function_body::skip_noise(text, index) {
            out.push(' ');
            index = end;
            continue;
        }
        let Some(character) = text[index..].chars().next() else {
            break;
        };
        out.push(character);
        index += character.len_utf8();
    }
    out
}

fn returns_clause(header: &str, word: &str) -> bool {
    let lower = header.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(index) = rest.find("returns") {
        let before = index == 0 || !is_ident_byte(&rest.as_bytes()[index - 1]);
        let after = rest[index + "returns".len()..].trim_start();
        if before && word_starts(after, word) {
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
