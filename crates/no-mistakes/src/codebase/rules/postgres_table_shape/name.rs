pub(super) fn normalize_function_name(name: &str) -> String {
    let last = last_identifier(name.trim());
    let Some(quoted) = last
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return last.to_ascii_lowercase();
    };
    quoted.replace("\"\"", "\"")
}

pub(super) fn last_identifier(name: &str) -> String {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = name.chars().peekable();
    while let Some(character) = chars.next() {
        if quoted {
            current.push(character);
            if character == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            }
            continue;
        }
        if character == '"' {
            quoted = true;
            current.push(character);
            continue;
        }
        if character == '.' {
            parts.push(std::mem::take(&mut current));
            continue;
        }
        current.push(character);
    }
    parts.push(current);
    parts
        .into_iter()
        .rev()
        .find(|part| !part.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_string()
}
