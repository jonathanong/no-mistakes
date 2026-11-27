pub fn parts(raw: &str) -> Vec<String> {
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

pub fn normalize_identifier(identifier: &str) -> String {
    let part = parts(identifier).pop().unwrap_or_default();
    if part.starts_with('"') && part.ends_with('"') && part.len() >= 2 {
        part[1..part.len() - 1].replace("\"\"", "\"")
    } else {
        part.to_ascii_lowercase()
    }
}

/// The parts of a SQL table name with their quoting removed: `other."Users"` is
/// `["other", "Users"]`, and `"audit.log"` is one part.
pub fn decoded_parts(name: &str) -> Vec<String> {
    parts(name)
        .iter()
        .map(|part| normalize_identifier(part))
        .collect()
}

pub fn normalize_table_name(table: &str) -> String {
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
pub fn plain_column(expression: &str) -> Option<String> {
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
