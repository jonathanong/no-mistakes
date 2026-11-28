pub(super) fn calls(line: &str) -> Vec<String> {
    call_spans(line).into_iter().map(|(_, callee)| callee).collect()
}

pub(super) fn per_item_callees(line: &str, construct: &str) -> Vec<String> {
    let body_at = body_offset(line);
    let iterable = construct == "foreach" || construct == "each" || is_for_in(line);
    call_spans(line)
        .into_iter()
        .filter(|(index, _)| body_at.is_some_and(|at| *index > at) || !iterable)
        .map(|(_, callee)| callee)
        .collect()
}

fn body_offset(line: &str) -> Option<usize> {
    if let Some(at) = line.find('{') {
        return Some(at);
    }
    if let Some(at) = line.find(" do") {
        return Some(at);
    }
    line.rfind(':').filter(|at| line[*at + 1..].trim().is_empty())
}

fn is_for_in(line: &str) -> bool {
    [" in ", " of ", " as ", " range ", " <- "]
        .iter()
        .any(|token| line.contains(token))
}

fn call_spans(line: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (index, byte) in line.as_bytes().iter().enumerate() {
        if *byte != b'(' {
            continue;
        }
        let callee = callee_before(line, index);
        if callee.is_empty() || is_keyword(&callee) {
            continue;
        }
        found.push((index, callee));
    }
    found
}

pub(crate) fn split_separator(callee: &str) -> Option<(&str, &str)> {
    let mut best = None;
    for (index, _) in callee.char_indices() {
        if callee[index..].starts_with('.') {
            best = Some((index, 1));
        } else if callee[index..].starts_with("::") || callee[index..].starts_with("->") {
            best = Some((index, 2));
        }
    }
    let (at, len) = best?;
    Some((&callee[..at], &callee[at + len..]))
}

pub(super) fn trailing_callee(line: &str) -> Option<&str> {
    let trimmed = line.trim_end();
    let bytes = trimmed.as_bytes();
    if bytes.last().is_none_or(|ch| !ch.is_ascii_alphanumeric() && *ch != b'_') {
        return None;
    }
    let mut start = bytes.len();
    while start > 0 && is_callee_byte(bytes[start - 1]) {
        start -= 1;
    }
    let name = trimmed[start..].trim_start_matches(['.', ':', '\\', '>', '-']);
    (!name.is_empty() && !is_keyword(name)).then_some(name)
}

fn callee_before(line: &str, paren: usize) -> String {
    let bytes = line.as_bytes();
    let mut end = paren;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_callee_byte(bytes[start - 1]) {
        start -= 1;
    }
    line[start..end]
        .trim_start_matches(['.', ':', '\\', '>', '-'])
        .to_string()
}

fn is_callee_byte(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'.' | b':' | b'\\' | b'$' | b'>' | b'-')
}

fn is_keyword(callee: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "for", "foreach", "while", "if", "switch", "catch", "func", "fn", "function", "def", "fun",
        "return", "elif", "else", "do", "end", "class", "new",
    ];
    let tail = split_separator(callee).map(|(_, tail)| tail).unwrap_or(callee);
    KEYWORDS.contains(&callee) || KEYWORDS.contains(&tail)
}
