use std::collections::BTreeSet;

/// A quoted flow scalar folded across lines is not one physical line.
/// Report it on the `uses` key rather than dropping it.
pub(super) fn claim_folded_uses(
    lines: &[&str],
    claimed: &mut BTreeSet<usize>,
    value: &str,
) -> Option<usize> {
    let head = value
        .split([' ', '\n'])
        .next()
        .filter(|head| !head.is_empty())?;
    for (index, line) in lines.iter().enumerate() {
        let line_num = index + 1;
        if claimed.contains(&line_num) {
            continue;
        }
        if folded_uses_key(line, head) {
            claimed.insert(line_num);
            return Some(line_num);
        }
    }
    None
}

fn folded_uses_key(line: &str, head: &str) -> bool {
    let Some(tail) = uses_key_tail(line) else {
        return false;
    };
    let tail = tail.trim_start();
    let Some(quote) = tail.chars().next().filter(|ch| *ch == '"' || *ch == '\'') else {
        return false;
    };
    let body = &tail[quote.len_utf8()..];
    body.contains(head) && !body.contains(quote)
}

fn uses_key_tail(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut quote = None;
    let mut index = 0;
    while index < bytes.len() {
        if !line.is_char_boundary(index) {
            index += 1;
            continue;
        }
        let ch = bytes[index];
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            let closer = if ch == b'"' { b"uses\"" } else { b"uses'" };
            if bytes[index + 1..].starts_with(closer) {
                let after_key = index + 1 + closer.len();
                let rest = line[after_key..].trim_start();
                if let Some(tail) = rest.strip_prefix(':') {
                    return Some(tail);
                }
            }
            quote = Some(ch);
            index += 1;
            continue;
        }
        if ch == b'#' {
            break;
        }
        if line[index..].starts_with("uses:") && uses_key_boundary(bytes, index) {
            return Some(&line[index + "uses:".len()..]);
        }
        index += 1;
    }
    None
}

fn uses_key_boundary(bytes: &[u8], index: usize) -> bool {
    index == 0 || !bytes[index - 1].is_ascii_alphanumeric()
}
