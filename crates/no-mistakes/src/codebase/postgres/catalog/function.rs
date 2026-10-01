use super::model::CatalogFunction;

pub(super) fn function_from_definition(key: &str, definition: &str) -> CatalogFunction {
    let (name, signature) = split_key(key);
    let (header, body) = super::function_body::split_body(definition);
    CatalogFunction {
        key: key.to_string(),
        name,
        signature,
        language: language(definition),
        returns_trigger: returns_trigger(&strip_quotes_and_comments(&header)),
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
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '-' && chars.peek() == Some(&'-') {
            chars.next();
            while chars.next().is_some_and(|next| next != '\n') {}
            out.push(' ');
            continue;
        }
        if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut depth = 1i32;
            while depth > 0 {
                match chars.next() {
                    Some('/') if chars.peek() == Some(&'*') => {
                        chars.next();
                        depth += 1;
                    }
                    Some('*') if chars.peek() == Some(&'/') => {
                        chars.next();
                        depth -= 1;
                    }
                    Some(_) => {}
                    None => break,
                }
            }
            out.push(' ');
            continue;
        }
        if character == '\'' || character == '"' {
            let quote = character;
            while let Some(next) = chars.next() {
                if next == quote {
                    if chars.peek() == Some(&quote) {
                        chars.next();
                        continue;
                    }
                    break;
                }
            }
            out.push(' ');
            continue;
        }
        out.push(character);
    }
    out
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
