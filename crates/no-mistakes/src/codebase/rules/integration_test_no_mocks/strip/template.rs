pub(super) fn push_erased(bytes: &[u8], start: usize, out: &mut String) -> usize {
    out.push(' ');
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => super::push_erased_escape(bytes, &mut index, out),
            b'`' => {
                out.push(' ');
                return index + 1;
            }
            b'$' if bytes.get(index + 1) == Some(&b'{') => {
                out.push_str("  ");
                index = push_expression(bytes, index + 2, out, true);
            }
            byte => {
                out.push(if byte == b'\n' { '\n' } else { ' ' });
                index += 1;
            }
        }
    }
    index
}

pub(super) fn push_preserved(bytes: &[u8], start: usize, out: &mut String) -> usize {
    out.push('`');
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => push_preserved_escape(bytes, &mut index, out),
            b'`' => {
                out.push('`');
                return index + 1;
            }
            b'$' if bytes.get(index + 1) == Some(&b'{') => {
                out.push_str("${");
                index = push_expression(bytes, index + 2, out, false);
            }
            byte => {
                out.push(byte as char);
                index += 1;
            }
        }
    }
    index
}

fn push_expression(bytes: &[u8], mut index: usize, out: &mut String, strings: bool) -> usize {
    let mut depth = 1usize;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = super::push_line_comment(bytes, index + 2, out);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = super::push_block_comment(bytes, index + 2, out);
            }
            b'/' if super::regex_literal::can_start(out) => {
                index = super::regex_literal::push_erased(bytes, index, out);
            }
            b'\'' | b'"' => {
                index = if strings {
                    super::push_erased_string(bytes, index, out)
                } else {
                    super::push_preserved_string(bytes, index, out)
                };
            }
            b'`' => {
                index = if strings {
                    push_erased(bytes, index, out)
                } else {
                    push_preserved(bytes, index, out)
                };
            }
            b'{' => {
                depth += 1;
                out.push('{');
                index += 1;
            }
            b'}' => {
                depth -= 1;
                out.push(if strings { ' ' } else { '}' });
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

fn push_preserved_escape(bytes: &[u8], index: &mut usize, out: &mut String) {
    out.push('\\');
    if let Some(next) = bytes.get(*index + 1) {
        out.push(*next as char);
        *index += 2;
    } else {
        *index += 1;
    }
}
