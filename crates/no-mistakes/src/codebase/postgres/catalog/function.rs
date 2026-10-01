use super::model::CatalogFunction;

pub(super) fn function_from_definition(key: &str, definition: &str) -> CatalogFunction {
    let (name, signature) = split_key(key);
    let (header, body, body_span) = super::function_body::split_body(definition);
    let plain = strip_quotes_and_comments(&header);
    CatalogFunction {
        key: key.to_string(),
        name,
        signature,
        language: language(definition),
        returns_trigger: returns_clause(&plain, "trigger"),
        returns_event_trigger: returns_clause(&plain, "event_trigger"),
        definition: definition.to_string(),
        body,
        null_input: null_input_mode(definition, body_span),
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

fn language(definition: &str) -> Option<String> {
    let bytes = definition.as_bytes();
    let mut index = 0;
    let mut depth = 0i32;
    while index < definition.len() {
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

fn null_input_mode(definition: &str, span: Option<(usize, usize)>) -> String {
    let text = blank_body(definition, span);
    let words = words_outside_literals(&text);
    let mut mode = "called";
    for index in 0..words.len() {
        if words[index] == "strict" {
            mode = "strict";
        } else if phrase_at(&words, index, &["called", "on", "null", "input"]) {
            mode = "called";
        } else if phrase_at(&words, index, &["returns", "null", "on", "null", "input"]) {
            mode = "strict";
        }
    }
    mode.to_string()
}

fn blank_body(definition: &str, span: Option<(usize, usize)>) -> String {
    let Some((start, end)) = span else {
        return definition.to_string();
    };
    if start > end || end > definition.len() {
        return definition.to_string();
    }
    let mut text = String::with_capacity(definition.len());
    text.push_str(&definition[..start]);
    text.push(' ');
    text.push_str(&definition[end..]);
    text
}

fn words_outside_literals(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut index = 0;
    let mut current = String::new();
    while index < text.len() {
        if let Some(end) = super::function_body::skip_noise(text, index) {
            push_word(&mut words, &mut current);
            index = end;
            continue;
        }
        let Some(character) = text[index..].chars().next() else {
            break;
        };
        if character.is_ascii_alphanumeric() || character == '_' {
            current.push(character.to_ascii_lowercase());
        } else {
            push_word(&mut words, &mut current);
        }
        index += character.len_utf8();
    }
    push_word(&mut words, &mut current);
    words
}

fn push_word(words: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        words.push(std::mem::take(current));
    }
}

fn phrase_at(words: &[String], index: usize, phrase: &[&str]) -> bool {
    words[index..]
        .iter()
        .map(String::as_str)
        .take(phrase.len())
        .eq(phrase.iter().copied())
}
