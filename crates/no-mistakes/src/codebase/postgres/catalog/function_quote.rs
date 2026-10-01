pub(super) fn body_after_as(definition: &str, cursor: usize) -> Option<String> {
    if let Some((tag, open_end)) = opening_dollar(definition, cursor) {
        let close = format!("${tag}$");
        let relative = definition[open_end..].find(&close)?;
        return Some(definition[open_end..open_end + relative].to_string());
    }
    quoted_sql_body(definition, cursor)
}

pub(super) fn opening_dollar(definition: &str, index: usize) -> Option<(String, usize)> {
    let rest = definition[index..].strip_prefix('$')?;
    let end = rest.find('$')?;
    Some((rest[..end].to_string(), index + end + 2))
}

fn quoted_sql_body(definition: &str, cursor: usize) -> Option<String> {
    if !definition[cursor..].starts_with('\'') {
        return None;
    }
    let end = super::function_body::skip_quoted(definition, cursor, '\'');
    if end <= cursor + 1 || definition.as_bytes().get(end - 1) != Some(&b'\'') {
        return None;
    }
    Some(unescape_sql_string(&definition[cursor + 1..end - 1]))
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
