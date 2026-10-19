pub(super) fn is_checkout_uses(uses: &str) -> bool {
    let uses = strip_comment(uses).trim();
    let uses = unquote(uses).trim();
    let name = match uses.split_once('@') {
        Some((name, _)) => name,
        None => uses,
    };
    // GitHub resolves the owner and repository without regard to case.
    name.trim_end_matches('/')
        .eq_ignore_ascii_case("actions/checkout")
}

fn strip_comment(value: &str) -> &str {
    let mut quote = None;
    for (index, ch) in value.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '#' {
            return &value[..index];
        }
    }
    value
}

fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}
