pub(super) fn scan_inline_arith_writes(
    raw: &str,
    on_write: &mut dyn FnMut(&str),
    mut scan: impl FnMut(&str, &mut dyn FnMut(&str)),
) {
    let bytes = raw.as_bytes();
    let (mut i, mut double_quoted) = (0, false);
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'\'' if !double_quoted => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
                i += 1;
            }
            b'"' => {
                double_quoted = !double_quoted;
                i += 1;
            }
            b'$' if bytes.get(i..i + 3) == Some(b"$((") => {
                let after = &raw[i + 3..];
                let Some(end) = matching_arith_end(after) else {
                    break;
                };
                scan(&after[..end], on_write);
                i += 3 + end + 2;
            }
            _ => i += 1,
        }
    }
}

fn matching_arith_end(body: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut depth = 0usize;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' if depth == 0 => {
                return (bytes.get(index + 1) == Some(&b')')).then_some(index);
            }
            b')' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
    None
}

pub(super) fn collect_substring_vars(raw: &str, mut i: usize, on_ident: &mut dyn FnMut(&str)) {
    let bytes = raw.as_bytes();
    if bytes.get(i) != Some(&b':')
        || matches!(bytes.get(i + 1), None | Some(b'-' | b'=' | b'+' | b'?'))
    {
        return;
    }
    i += 1;
    let start = i;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'{' {
            depth += 1;
        } else if bytes[i] == b'}' {
            depth -= 1;
        }
        if depth > 0 {
            i += 1;
        }
    }
    if depth != 0 {
        return;
    }
    let expr = &raw[start..i];
    let ebytes = expr.as_bytes();
    let mut j = 0;
    while j < ebytes.len() {
        if ebytes[j] == b'_' || ebytes[j].is_ascii_alphabetic() {
            let s = j;
            while j < ebytes.len() && (ebytes[j] == b'_' || ebytes[j].is_ascii_alphanumeric()) {
                j += 1;
            }
            on_ident(&expr[s..j]);
        } else {
            j += 1;
        }
    }
}
