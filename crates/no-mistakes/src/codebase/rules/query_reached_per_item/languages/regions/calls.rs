pub(super) fn calls(line: &str) -> Vec<String> {
    call_spans(line)
        .into_iter()
        .map(|(_, callee)| callee)
        .collect()
}

pub(super) fn per_item_callees(line: &str, construct: &str) -> Vec<String> {
    let body_at = body_offset(line);
    let iterable = construct == "foreach"
        || construct == "each"
        || is_for_in(line)
        || (construct == "for" && line.contains(" : ") && !line.contains(';'));
    let init_end = c_style_init_end(line, construct);
    call_spans(line)
        .into_iter()
        .filter(|(index, _)| init_end.is_none_or(|end| *index >= end))
        .filter(|(index, _)| body_at.is_some_and(|at| *index > at) || !iterable)
        .map(|(_, callee)| callee)
        .collect()
}

pub(super) fn initializer_callees(line: &str, construct: &str) -> Vec<String> {
    let Some(end) = c_style_init_end(line, construct) else {
        return Vec::new();
    };
    call_spans(line)
        .into_iter()
        .filter(|(index, _)| *index < end)
        .map(|(_, callee)| callee)
        .collect()
}

pub(super) fn carried_calls(
    joined: &str,
    split_at: usize,
    carried_line: u32,
    line_no: u32,
) -> Vec<(u32, String)> {
    let first = split_at
        + joined
            .get(split_at..)
            .and_then(|rest| rest.find('('))
            .unwrap_or(0);
    call_spans(joined)
        .into_iter()
        .map(|(index, callee)| {
            let line = if index == first {
                carried_line
            } else {
                line_no
            };
            (line, callee)
        })
        .collect()
}

fn body_offset(line: &str) -> Option<usize> {
    if let Some(at) = line.find('{') {
        return Some(at);
    }
    if let Some(at) = line.find(" do") {
        return Some(at);
    }
    line.rfind(':')
        .filter(|at| line[*at + 1..].trim().is_empty())
}

fn is_for_in(line: &str) -> bool {
    [" in ", " of ", " as ", " range ", " <- "]
        .iter()
        .any(|token| line.contains(token))
}

fn c_style_init_end(line: &str, construct: &str) -> Option<usize> {
    if construct != "for" || is_for_in(line) {
        return None;
    }
    line.find(';')
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
    if bytes
        .last()
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && *ch != b'_')
    {
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
    end = skip_generics(bytes, end);
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_callee_byte(bytes[start - 1]) {
        start -= 1;
    }
    line[start..end]
        .trim_start_matches(['.', ':', '\\', '>', '-'])
        .trim_end_matches(':')
        .to_string()
}

fn skip_generics(bytes: &[u8], end: usize) -> usize {
    if end == 0 || bytes[end - 1] != b'>' {
        return end;
    }
    let mut depth = 0i32;
    let mut index = end;
    while index > 0 {
        index -= 1;
        match bytes[index] {
            b'>' => depth += 1,
            b'<' => {
                depth -= 1;
                if depth == 0 {
                    let attached = index > 0
                        && (bytes[index - 1].is_ascii_alphanumeric()
                            || matches!(bytes[index - 1], b'_' | b':'));
                    return if attached { index } else { end };
                }
            }
            _ => {}
        }
    }
    end
}

pub(super) fn is_callee_byte(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'.' | b':' | b'\\' | b'$' | b'>' | b'-')
}

pub(super) fn is_keyword(callee: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "for", "foreach", "while", "if", "switch", "catch", "func", "fn", "function", "def", "fun",
        "return", "elif", "else", "do", "end", "class", "new",
    ];
    split_separator(callee).is_none() && KEYWORDS.contains(&callee)
}
