mod regex_literal;

pub(super) fn comments_and_regex_literals(content: &str) -> String {
    strip(content, false, true)
}

pub(super) fn strip(content: &str, strings: bool, regex_literals: bool) -> String {
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
            b'\'' | b'"' => {
                index = if strings {
                    push_erased_string(bytes, index, &mut out)
                } else {
                    push_preserved_string(bytes, index, &mut out)
                };
            }
            b'`' => {
                index = if strings {
                    push_erased_template(bytes, index, &mut out)
                } else {
                    push_preserved_string(bytes, index, &mut out)
                };
            }
            b'/' if regex_literals && regex_literal::can_start(&out) => {
                index = regex_literal::push_erased(bytes, index, &mut out);
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

fn push_erased_string(bytes: &[u8], start: usize, out: &mut String) -> usize {
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

fn push_erased_template(bytes: &[u8], start: usize, out: &mut String) -> usize {
    out.push(' ');
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                push_erased_escape(bytes, &mut index, out);
            }
            b'`' => {
                out.push(' ');
                return index + 1;
            }
            b'$' if bytes.get(index + 1) == Some(&b'{') => {
                out.push_str("  ");
                index = push_template_expression(bytes, index + 2, out);
            }
            byte => {
                out.push(if byte == b'\n' { '\n' } else { ' ' });
                index += 1;
            }
        }
    }
    index
}

fn push_template_expression(bytes: &[u8], mut index: usize, out: &mut String) -> usize {
    let mut depth = 1usize;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = push_line_comment(bytes, index + 2, out);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = push_block_comment(bytes, index + 2, out);
            }
            b'/' if regex_literal::can_start(out) => {
                index = regex_literal::push_erased(bytes, index, out);
            }
            b'\'' | b'"' => {
                index = push_erased_string(bytes, index, out);
            }
            b'`' => {
                index = push_erased_template(bytes, index, out);
            }
            b'{' => {
                depth += 1;
                out.push('{');
                index += 1;
            }
            b'}' => {
                depth -= 1;
                out.push(' ');
                index += 1;
                if depth == 0 {
                    return index;
                }
            }
            byte => {
                out.push(byte as char);
                index += 1;
            }
        }
    }
    index
}

fn push_erased_escape(bytes: &[u8], index: &mut usize, out: &mut String) {
    out.push(' ');
    if let Some(next) = bytes.get(*index + 1) {
        out.push(if *next == b'\n' { '\n' } else { ' ' });
        *index += 2;
    } else {
        *index += 1;
    }
}

fn push_preserved_string(bytes: &[u8], start: usize, out: &mut String) -> usize {
    let quote = bytes[start];
    out.push(quote as char);
    let mut index = start + 1;
    while index < bytes.len() {
        let byte = bytes[index];
        out.push(byte as char);
        if byte == b'\\' {
            if let Some(next) = bytes.get(index + 1) {
                out.push(*next as char);
                index += 2;
            } else {
                index += 1;
            }
        } else if byte == quote {
            return index + 1;
        } else {
            index += 1;
        }
    }
    index
}
