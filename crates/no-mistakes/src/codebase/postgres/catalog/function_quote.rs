pub(super) fn body_after_as(definition: &str, cursor: usize) -> Option<(String, usize, usize)> {
    let (body, start, end, next) = one_literal(definition, cursor)?;
    Some(append_adjacent(definition, body, start, end, next))
}

pub(super) fn return_expression(definition: &str) -> Option<(String, String, (usize, usize))> {
    let bytes = definition.as_bytes();
    let mut index = 0;
    let mut depth = 0i32;
    while index < definition.len() {
        if let Some(next) = super::function_body::skip_noise(definition, index) {
            index = next;
            continue;
        }
        match bytes.get(index) {
            Some(b'(') => depth += 1,
            Some(b')') => depth -= 1,
            _ => {}
        }
        if depth == 0 && super::function_body::is_word_at(definition, index, "return") {
            let start = index + "return".len();
            return Some((
                definition[..index].to_string(),
                definition[start..].to_string(),
                (start, definition.len()),
            ));
        }
        index += definition[index..].chars().next()?.len_utf8();
    }
    None
}

fn one_literal(definition: &str, cursor: usize) -> Option<(String, usize, usize, usize)> {
    if let Some(literal) = unicode_literal(definition, cursor) {
        return Some(literal);
    }
    if let Some((tag, open_end)) = opening_dollar(definition, cursor) {
        let close = format!("${tag}$");
        let relative = definition[open_end..].find(&close)?;
        let end = open_end + relative;
        return Some((
            definition[open_end..end].to_string(),
            open_end,
            end,
            end + close.len(),
        ));
    }
    let (quote_at, escape) = quote_at(definition, cursor)?;
    let (body, start, end) = if escape {
        escape_string_body(definition, quote_at)?
    } else {
        quoted_sql_body(definition, quote_at)?
    };
    Some((body, start, end, end + 1))
}

fn append_adjacent(
    definition: &str,
    mut body: String,
    start: usize,
    mut end: usize,
    mut cursor: usize,
) -> (String, usize, usize) {
    loop {
        cursor = super::function_body::skip_as_gap(definition, cursor);
        let Some((more, _, more_end, next)) = one_literal(definition, cursor) else {
            break;
        };
        if next <= cursor {
            break;
        }
        body.push_str(&more);
        end = more_end;
        cursor = next;
    }
    (body, start, end)
}

fn escape_string_body(definition: &str, quote_at: usize) -> Option<(String, usize, usize)> {
    let mut index = quote_at + 1;
    while index < definition.len() {
        let character = definition[index..].chars().next()?;
        let len = character.len_utf8();
        if character == '\\' {
            index += len;
            if let Some(escaped) = definition[index..].chars().next() {
                index += escaped.len_utf8();
            }
            continue;
        }
        if character == '\'' {
            if definition[index + len..].starts_with('\'') {
                index += len + 1;
                continue;
            }
            let start = quote_at + 1;
            return Some((
                super::function_escape::unescape(&definition[start..index]),
                start,
                index,
            ));
        }
        index += len;
    }
    None
}

fn unicode_literal(definition: &str, cursor: usize) -> Option<(String, usize, usize, usize)> {
    let rest = definition.get(cursor..)?;
    let prefix = rest.chars().next()?;
    if !prefix.eq_ignore_ascii_case(&'u') {
        return None;
    }
    let after = cursor + prefix.len_utf8();
    if !definition.get(after..)?.starts_with("&'") {
        return None;
    }
    let (raw, start, end) = quoted_sql_body(definition, after + 1)?;
    let escape = unicode_escape_character(definition, end + 1)?;
    Some((
        super::function_escape::unescape_unicode_with_escape(&raw, escape),
        start,
        end,
        end + 1,
    ))
}

fn unicode_escape_character(definition: &str, cursor: usize) -> Option<char> {
    let cursor = super::function_body::skip_as_gap(definition, cursor);
    if !super::function_body::is_word_at(definition, cursor, "uescape") {
        return Some('\\');
    }
    let cursor = super::function_body::skip_as_gap(definition, cursor + "uescape".len());
    let (value, _, _) = quoted_sql_body(definition, cursor)?;
    let mut chars = value.chars();
    let escape = chars.next()?;
    (chars.next().is_none()
        && !escape.is_ascii_hexdigit()
        && !matches!(escape, '+' | '\'' | '"')
        && !escape.is_whitespace())
    .then_some(escape)
}

fn quote_at(definition: &str, cursor: usize) -> Option<(usize, bool)> {
    let rest = definition.get(cursor..)?;
    if rest.starts_with('\'') {
        return Some((cursor, false));
    }
    let mut chars = rest.chars();
    let prefix = chars.next()?;
    if prefix.eq_ignore_ascii_case(&'e') && chars.next() == Some('\'') {
        return Some((cursor + prefix.len_utf8(), true));
    }
    None
}

pub(super) fn opening_dollar(definition: &str, index: usize) -> Option<(String, usize)> {
    if let Some(before) = definition[..index].chars().next_back() {
        if before.is_alphanumeric() || before == '_' {
            return None;
        }
    }
    let rest = definition[index..].strip_prefix('$')?;
    let end = rest.find('$')?;
    let tag = &rest[..end];
    let valid = tag.is_empty()
        || (tag
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
            && tag.chars().all(|c| c.is_alphanumeric() || c == '_'));
    valid.then(|| (tag.to_string(), index + end + 2))
}

pub(super) fn quoted_sql_body(definition: &str, cursor: usize) -> Option<(String, usize, usize)> {
    if !definition[cursor..].starts_with('\'') {
        return None;
    }
    let end = super::function_comment::skip_quoted(definition, cursor, '\'');
    if end <= cursor + 1 || definition.as_bytes().get(end - 1) != Some(&b'\'') {
        return None;
    }
    let start = cursor + 1;
    Some((
        unescape_sql_string(&definition[start..end - 1]),
        start,
        end - 1,
    ))
}

fn unescape_sql_string(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' && chars.peek() == Some(&'\'') {
            chars.next();
        }
        out.push(character);
    }
    out
}
