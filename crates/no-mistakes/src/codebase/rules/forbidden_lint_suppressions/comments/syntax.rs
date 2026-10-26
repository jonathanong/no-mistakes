pub(super) fn header(text: &str, directive: &str) -> bool {
    text.strip_prefix(directive)
        .is_some_and(|tail| tail.is_empty() || tail.starts_with(char::is_whitespace))
}

pub(super) fn typescript_text(raw: &str, block: bool) -> &str {
    if block {
        // TypeScript scans the last line of a block, including JSDoc prefixes.
        raw.rsplit(['\n', '\r', '\u{2028}', '\u{2029}'])
            .next()
            .unwrap_or(raw)
            .trim_start()
            .trim_start_matches(['*', '/'])
            .trim_start()
    } else {
        raw.strip_prefix('/').unwrap_or(raw).trim_start()
    }
}

pub(super) fn rule_id(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

pub(super) fn without_reason(text: &str) -> &str {
    // ESLint's justification separator is whitespace, 2+ hyphens, whitespace.
    // Hyphens within an actual rule identifier must remain part of its name.
    for (at, c) in text.char_indices() {
        if !c.is_whitespace() {
            continue;
        }
        let tail = &text[at + c.len_utf8()..];
        let hyphens = tail.bytes().take_while(|v| *v == b'-').count();
        if hyphens >= 2 && tail[hyphens..].starts_with(char::is_whitespace) {
            return &text[..at];
        }
    }
    text
}
