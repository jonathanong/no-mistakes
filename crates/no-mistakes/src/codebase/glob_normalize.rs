pub(crate) fn normalize(pattern: &str) -> String {
    let mut parts = Vec::new();
    for part in pattern.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(is_literal_segment) => {
                parts.pop();
            }
            ".." => parts.push(part),
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn is_literal_segment(segment: &&str) -> bool {
    !segment
        .chars()
        .any(|ch| matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\'))
}
