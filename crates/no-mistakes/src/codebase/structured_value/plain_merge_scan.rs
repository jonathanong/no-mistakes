/// Returns `true` when `source` contains any unquoted, uncommented plain `<<`
/// token in a position that could act as a YAML 1.1 merge key (direct mapping
/// key, explicit `? <<` key, or anchored `&name <<` scalar).
pub(super) fn has_plain_merge_key(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut line_start = true;
    let mut line_indent = 0usize;
    let mut block_scalar_indent: Option<usize> = None;

    while i < bytes.len() {
        if line_start {
            if block_scalar_indent.is_none() && bytes[i] == b'%' {
                i = skip_to_newline(bytes, i);
                continue;
            }
            let spaces = count_leading_spaces(bytes, i);
            let after_spaces = i + spaces;
            if let Some(hdr_indent) = block_scalar_indent {
                if after_spaces >= bytes.len()
                    || matches!(bytes[after_spaces], b'\r' | b'\n')
                    || spaces > hdr_indent
                {
                    i = skip_to_newline(bytes, after_spaces);
                    continue;
                }
                block_scalar_indent = None;
            }
            line_indent = spaces;
            i = after_spaces;
            line_start = false;
            continue;
        }

        match bytes[i] {
            b'\n' => {
                i += 1;
                line_start = true;
            }
            b' ' | b'\t' | b'\r' => i += 1,
            b'#' if i == 0 || matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r') => {
                i = skip_to_newline(bytes, i);
                line_start = true;
            }
            b'\'' => i = skip_single_quoted(bytes, i),
            b'"' => i = skip_double_quoted(bytes, i),
            b'|' | b'>' if starts_block_scalar(bytes, i) => {
                block_scalar_indent = Some(line_indent);
                i = skip_to_newline(bytes, i);
                line_start = true;
            }
            b'<' if i + 1 < bytes.len() && bytes[i + 1] == b'<' => {
                if is_plain_merge_candidate(bytes, i) {
                    return true;
                }
                i += 2;
            }
            _ => i += 1,
        }
    }
    false
}

fn count_leading_spaces(bytes: &[u8], start: usize) -> usize {
    let mut n = 0;
    while start + n < bytes.len() && bytes[start + n] == b' ' {
        n += 1;
    }
    n
}

fn skip_to_newline(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        let byte = bytes[i];
        i += 1;
        if byte == b'\n' {
            break;
        }
    }
    i
}

fn skip_single_quoted(bytes: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < bytes.len() {
        if bytes[i] == b'\'' {
            i += 1;
            if i < bytes.len() && bytes[i] == b'\'' {
                i += 1;
                continue;
            }
            break;
        }
        i += 1;
    }
    i
}

fn skip_double_quoted(bytes: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        if bytes[i] == b'"' {
            i += 1;
            break;
        }
        i += 1;
    }
    i
}

fn starts_block_scalar(bytes: &[u8], pos: usize) -> bool {
    if pos > 0 && !matches!(bytes[pos - 1], b' ' | b'\t' | b':' | b'-' | b'?') {
        return false;
    }
    let mut i = pos + 1;
    while i < bytes.len() && matches!(bytes[i], b'-' | b'+' | b'1'..=b'9') {
        i += 1;
    }
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    i >= bytes.len() || matches!(bytes[i], b'\r' | b'\n' | b'#')
}

fn is_plain_merge_candidate(bytes: &[u8], pos: usize) -> bool {
    let left_ok = pos == 0
        || matches!(
            bytes[pos - 1],
            b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'[' | b',' | b'-' | b'?' | b':'
        );
    let after = pos + 2;
    let right_ok = after >= bytes.len()
        || matches!(
            bytes[after],
            b' ' | b'\t' | b'\n' | b'\r' | b':' | b',' | b'}' | b']' | b'#'
        );
    if !left_ok || !right_ok {
        return false;
    }
    let prev = previous_token_on_line(bytes, pos);
    if prev.is_some_and(|token| token.starts_with(b"!")) {
        return false;
    }
    if followed_by_mapping_colon(bytes, after) {
        return true;
    }
    prev.is_some_and(|token| token == b"?" || (token.starts_with(b"&") && token.len() > 1))
}

fn followed_by_mapping_colon(bytes: &[u8], mut i: usize) -> bool {
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b':' {
        return false;
    }
    let next = i + 1;
    next >= bytes.len()
        || matches!(
            bytes[next],
            b' ' | b'\t' | b'\n' | b'\r' | b',' | b'}' | b']'
        )
}

fn previous_token_on_line(bytes: &[u8], pos: usize) -> Option<&[u8]> {
    let mut end = pos;
    while end > 0 && matches!(bytes[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    if end == 0 || matches!(bytes[end - 1], b'\n' | b'\r' | b'{' | b'[' | b',' | b':') {
        return None;
    }
    let mut start = end;
    while start > 0
        && !matches!(
            bytes[start - 1],
            b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'[' | b',' | b':'
        )
    {
        start -= 1;
    }
    Some(&bytes[start..end])
}
