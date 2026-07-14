pub(super) fn glob_escape_literal(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\') {
                vec!['\\', ch]
            } else {
                vec![ch]
            }
        })
        .collect()
}

pub(super) fn normalize_glob_template(pattern: &str) -> String {
    normalize_relative_pattern(pattern)
}

pub(super) fn normalize_relative_pattern(pattern: &str) -> String {
    let mut parts = Vec::new();
    for part in pattern.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

pub(super) fn is_declaration_file(rel: &str) -> bool {
    rel.ends_with(".d.ts") || rel.ends_with(".d.mts") || rel.ends_with(".d.cts")
}
