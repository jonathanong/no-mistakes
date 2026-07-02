pub(super) fn mask(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let marker_len = count_backticks(bytes, index);
            if let Some(end) = find_close(bytes, index + marker_len, marker_len) {
                let span = &source[index..end];
                if span.contains("<!--") {
                    push_masked(span, &mut out);
                } else {
                    out.push_str(span);
                }
                index = end;
            } else {
                out.push('`');
                index += 1;
            }
        } else {
            out.push(bytes[index] as char);
            index += 1;
        }
    }
    out
}

fn find_close(bytes: &[u8], mut index: usize, marker_len: usize) -> Option<usize> {
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let close_len = count_backticks(bytes, index);
            if close_len == marker_len {
                return Some(index + close_len);
            }
            index += close_len;
        } else {
            index += 1;
        }
    }
    None
}

fn count_backticks(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .take_while(|byte| **byte == b'`')
        .count()
}

fn push_masked(value: &str, out: &mut String) {
    out.extend(value.chars().map(|ch| if ch == '\n' { '\n' } else { 'x' }));
}
