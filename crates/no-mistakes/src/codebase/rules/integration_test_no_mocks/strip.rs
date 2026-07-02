pub(super) fn comments(content: &str) -> String {
    strip(content, false)
}

pub(super) fn comments_and_strings(content: &str) -> String {
    strip(content, true)
}

fn strip(content: &str, strings: bool) -> String {
    let mut out = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = push_line_comment(bytes, index + 2, &mut out);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = push_block_comment(bytes, index + 2, &mut out);
            }
            b'\'' | b'"' | b'`' if strings => {
                index = push_string(bytes, index, &mut out);
            }
            byte => {
                out.push(byte as char);
                index += 1;
            }
        }
    }
    out
}

fn push_line_comment(bytes: &[u8], mut index: usize, out: &mut String) -> usize {
    out.push_str("  ");
    while index < bytes.len() && bytes[index] != b'\n' {
        out.push(' ');
        index += 1;
    }
    if index < bytes.len() {
        out.push('\n');
        index += 1;
    }
    index
}

fn push_block_comment(bytes: &[u8], mut index: usize, out: &mut String) -> usize {
    out.push_str("  ");
    while index < bytes.len() {
        if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
            out.push_str("  ");
            return index + 2;
        }
        out.push(if bytes[index] == b'\n' { '\n' } else { ' ' });
        index += 1;
    }
    index
}

fn push_string(bytes: &[u8], start: usize, out: &mut String) -> usize {
    let quote = bytes[start];
    out.push(' ');
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            out.push(' ');
            if let Some(next) = bytes.get(index + 1) {
                out.push(if *next == b'\n' { '\n' } else { ' ' });
                index += 2;
            } else {
                index += 1;
            }
        } else if bytes[index] == quote {
            out.push(' ');
            return index + 1;
        } else {
            out.push(if bytes[index] == b'\n' { '\n' } else { ' ' });
            index += 1;
        }
    }
    index
}
