pub(super) fn body_after_as(definition: &str, cursor: usize) -> Option<(String, usize, usize)> {
    if let Some((tag, open_end)) = opening_dollar(definition, cursor) {
        let close = format!("${tag}$");
        let relative = definition[open_end..].find(&close)?;
        let end = open_end + relative;
        return Some((definition[open_end..end].to_string(), open_end, end));
    }
    let (quote_at, escape) = quote_at(definition, cursor)?;
    if escape {
        return escape_string_body(definition, quote_at);
    }
    quoted_sql_body(definition, quote_at)
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
                unescape_escape_string(&definition[start..index]),
                start,
                index,
            ));
        }
        index += len;
    }
    None
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

fn unescape_escape_string(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\'' && chars.peek() == Some(&'\'') {
            chars.next();
            out.push('\'');
            continue;
        }
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000c}'),
            Some('\\') => out.push('\\'),
            Some('\'') => out.push('\''),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

pub(super) fn opening_dollar(definition: &str, index: usize) -> Option<(String, usize)> {
    let rest = definition[index..].strip_prefix('$')?;
    let end = rest.find('$')?;
    Some((rest[..end].to_string(), index + end + 2))
}

fn quoted_sql_body(definition: &str, cursor: usize) -> Option<(String, usize, usize)> {
    if !definition[cursor..].starts_with('\'') {
        return None;
    }
    let end = super::function_body::skip_quoted(definition, cursor, '\'');
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
