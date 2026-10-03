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
