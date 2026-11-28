pub(super) fn words(line: &str) -> Vec<&str> {
    line.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|word| !word.is_empty())
        .collect()
}

pub(super) fn standalone_words(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !(bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_') {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
        {
            index += 1;
        }
        let qualified = start > 0 && matches!(bytes[start - 1], b'.' | b':' | b'@' | b'$');
        let keyword_key =
            index < bytes.len() && bytes[index] == b':' && bytes.get(index + 1) != Some(&b':');
        if !qualified && !keyword_key {
            found.push(&line[start..index]);
        }
    }
    found
}

pub(super) fn top_level_colon(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut depth = 0i32;
    for (index, byte) in bytes.iter().enumerate() {
        match *byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = (depth - 1).max(0),
            b':' if depth == 0
                && bytes.get(index.wrapping_sub(1)) != Some(&b':')
                && bytes.get(index + 1) != Some(&b':') =>
            {
                return Some(index);
            }
            _ => {}
        }
    }
    None
}

pub(super) fn matching_brace_close(line: &str, open: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    for (offset, byte) in bytes[open..].iter().enumerate() {
        match *byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
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
