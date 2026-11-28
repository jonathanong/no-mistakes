pub(super) fn words(line: &str) -> Vec<&str> {
    line.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|word| !word.is_empty())
        .collect()
}

pub(super) fn first_word(trimmed: &str) -> &str {
    trimmed
        .split(|ch: char| ch.is_whitespace() || ch == '(' || ch == '{')
        .next()
        .unwrap_or("")
}

pub(super) fn leading_ident(rest: &str) -> &str {
    let end = rest
        .char_indices()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_')
        .map(|(index, _)| index)
        .unwrap_or(rest.len());
    &rest[..end]
}

pub(super) fn last_ident(before: &str) -> String {
    let mut last = String::new();
    let mut current = String::new();
    for ch in before.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            last = std::mem::take(&mut current);
        }
    }
    if current.is_empty() {
        last
    } else {
        current
    }
}
