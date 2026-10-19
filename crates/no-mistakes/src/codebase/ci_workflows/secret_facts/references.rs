use super::Reference;

pub(super) fn collect(text: &str, bare: bool, line: usize) -> Vec<Reference> {
    let mut found = Vec::new();
    if bare {
        scan(text, line, &mut found);
    } else {
        let mut rest = text;
        let mut consumed = 0;
        while let Some(start) = rest.find("${{") {
            let body = start + 3;
            let Some(end) = expression_end(&rest[body..]) else {
                found.push(Reference {
                    name: None,
                    line: line + text[..consumed + start].matches('\n').count(),
                });
                break;
            };
            let prefix = &text[..consumed + body];
            scan(
                &rest[body..body + end],
                line + prefix.matches('\n').count(),
                &mut found,
            );
            consumed += body + end + 2;
            rest = &text[consumed..];
        }
    }
    found
}

fn expression_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if matches!(bytes[at], b'\'' | b'"') {
            at = quoted_end(bytes, at);
        } else if bytes.get(at..at + 2) == Some(b"}}") {
            return Some(at);
        } else {
            at += 1;
        }
    }
    None
}

fn scan(text: &str, line: usize, found: &mut Vec<Reference>) {
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if matches!(bytes[at], b'\'' | b'"') {
            at = quoted_end(bytes, at);
            continue;
        }
        if !bytes
            .get(at..at + 7)
            .is_some_and(|part| part.eq_ignore_ascii_case(b"secrets"))
            || at
                .checked_sub(1)
                // Step/job ids such as `load-secrets` contain hyphens.
                .is_some_and(|before| {
                    identifier(bytes[before]) || matches!(bytes[before], b'.' | b'-')
                })
            || bytes.get(at + 7).is_some_and(|byte| identifier(*byte))
        {
            at += 1;
            continue;
        }
        let start = at;
        at += 7;
        let access = static_access(text, at);
        let name = access.as_ref().map(|(name, _)| name.clone());
        if let Some((_, end)) = access {
            at = end;
        }
        found.push(Reference {
            name,
            line: line + text[..start].matches('\n').count(),
        });
    }
}

fn static_access(text: &str, mut at: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    skip_space(bytes, &mut at);
    if bytes.get(at) == Some(&b'.') {
        at += 1;
        skip_space(bytes, &mut at);
        let start = at;
        while bytes.get(at).is_some_and(|byte| identifier(*byte)) {
            at += 1;
        }
        return (at > start).then(|| (text[start..at].to_owned(), at));
    }
    if bytes.get(at) != Some(&b'[') {
        return None;
    }
    at += 1;
    skip_space(bytes, &mut at);
    // GitHub expression strings are single-quoted. Double quotes throw, so
    // `secrets["FOO"]` is not the static name FOO.
    // https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#literals
    // "Wrapping with double quotes (`"`) will throw an error."
    if bytes.get(at) != Some(&b'\'') {
        return None;
    }
    let end = quoted_end(bytes, at);
    let name = text.get(at + 1..end.checked_sub(1)?)?;
    if name.is_empty() {
        return None;
    }
    at = end;
    skip_space(bytes, &mut at);
    (bytes.get(at) == Some(&b']')).then(|| (name.to_owned(), at + 1))
}

fn skip_space(bytes: &[u8], at: &mut usize) {
    while bytes.get(*at).is_some_and(u8::is_ascii_whitespace) {
        *at += 1;
    }
}

fn identifier(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn quoted_end(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut at = start + 1;
    while at < bytes.len() {
        if bytes[at] == quote {
            if bytes.get(at + 1) == Some(&quote) {
                at += 2;
            } else {
                return at + 1;
            }
        } else {
            at += 1;
        }
    }
    bytes.len()
}
