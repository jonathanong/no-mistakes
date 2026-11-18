use super::{finding, line_scan::builtin_pins, line_scan::Found, CompiledPattern};

pub(super) fn applies(file: &str, pattern: &CompiledPattern) -> bool {
    pattern.builtin
        && pattern.reason == "exact action ref"
        && matches!(
            file.rsplit_once('.').map(|(_, extension)| extension),
            Some("js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts")
        )
}

pub(super) fn scan(
    file: &str,
    content: &str,
    pattern: &CompiledPattern,
    ranges: &[(usize, usize)],
    found: &mut Vec<Found>,
) {
    if !applies(file, pattern) {
        return;
    }
    let mut offset = 0;
    let mut line = 1;
    for &(start, end) in ranges {
        line += content[offset..start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        offset = start;
        let Some(decoded) = literal(&content[start..end]) else {
            continue;
        };
        for pin in builtin_pins(&decoded, pattern) {
            found.push(Found::whole(finding(file, line, pattern, pin.as_str())));
        }
    }
}

fn literal(pattern: &str) -> Option<String> {
    let pattern = pattern.strip_prefix('^').unwrap_or(pattern);
    let pattern = pattern.strip_suffix('$').unwrap_or(pattern);
    let mut decoded = String::with_capacity(pattern.len());
    let mut chars = pattern.chars();
    while let Some(character) = chars.next() {
        match character {
            '\\' => match chars.next()? {
                '/' => decoded.push('/'),
                '.' => decoded.push('.'),
                _ => return None,
            },
            '\n' | '\r' | '\u{2028}' | '\u{2029}' => return None,
            '.' | '^' | '$' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
                return None;
            }
            _ => decoded.push(character),
        }
    }
    Some(decoded)
}
