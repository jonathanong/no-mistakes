fn parts(raw: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut part = String::new();
    let mut quoted = false;
    let mut chars = raw.trim().chars().peekable();
    while let Some(character) = chars.next() {
        if character == '"' {
            part.push(character);
            if quoted && chars.peek() == Some(&'"') {
                part.push(chars.next().unwrap());
            } else {
                quoted = !quoted;
            }
        } else if character == '.' && !quoted {
            parts.push(part.trim().to_string());
            part.clear();
        } else {
            part.push(character);
        }
    }
    parts.push(part.trim().to_string());
    parts
}

pub(super) fn normalize_identifier(identifier: &str) -> String {
    let part = parts(identifier).pop().unwrap_or_default();
    if part.starts_with('"') && part.ends_with('"') && part.len() >= 2 {
        part[1..part.len() - 1].replace("\"\"", "\"")
    } else {
        part.to_ascii_lowercase()
    }
}

/// The parts of a SQL table name with their quoting removed: `other."Users"` is
/// `["other", "Users"]`, and `"audit.log"` is one part.
pub(crate) fn decoded_parts(name: &str) -> Vec<String> {
    parts(name)
        .iter()
        .map(|part| normalize_identifier(part))
        .collect()
}

/// A SQL name split into its schema qualifier, if it has one, and its bare name, both decoded
/// the way PostgreSQL reads them (an unquoted part folds to lower case):
/// `public."audit.log"` is `(Some("public"), "audit.log")`.
pub(super) fn split_name(name: &str) -> (Option<String>, String) {
    split(decoded_parts(name))
}

/// The same for a key of the catalog, which is already normalized: its parts are unquoted but
/// never case-folded again, since a mixed-case key (`Order Items`) is exact.
pub(super) fn split_key(key: &str) -> (Option<String>, String) {
    let unquote = |part: &String| match part.strip_prefix('"').and_then(|p| p.strip_suffix('"')) {
        Some(inner) => inner.replace("\"\"", "\""),
        None => part.clone(),
    };
    split(parts(key).iter().map(unquote).collect())
}

fn split(mut parts: Vec<String>) -> (Option<String>, String) {
    let bare = parts.pop().unwrap_or_default();
    ((!parts.is_empty()).then(|| parts.join(".")), bare)
}

pub(crate) fn normalize_table_name(table: &str) -> String {
    parts(table)
        .iter()
        .map(|part| {
            let value = normalize_identifier(part);
            if value.contains(['.', '"']) {
                format!("\"{}\"", value.replace('"', "\"\""))
            } else {
                value
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// The column name when an index key expression is a bare (possibly quoted) identifier.
pub(super) fn plain_column(expression: &str) -> Option<String> {
    let text = expression.trim();
    let bare = text
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    let quoted = text.len() >= 2
        && text.starts_with('"')
        && text.ends_with('"')
        && !text[1..text.len() - 1].replace("\"\"", "").contains('"');
    (bare || quoted).then(|| normalize_identifier(text))
}
