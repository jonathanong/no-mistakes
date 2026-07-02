pub(super) fn end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) == Some(&b'<') {
        return angle_end(bytes, start + 1);
    }
    let mut index = start;
    let mut paren_depth = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            b'\'' | b'"' => {
                index = skip_quoted_title(bytes, index)?;
            }
            b'(' => {
                paren_depth += 1;
                index += 1;
            }
            b')' if paren_depth == 0 => return Some(index),
            b')' => {
                paren_depth -= 1;
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}

fn skip_quoted_title(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == quote => return Some(index + 1),
            _ => index += 1,
        }
    }
    None
}

fn angle_end(bytes: &[u8], start: usize) -> Option<usize> {
    let end = bytes[start..].iter().position(|byte| *byte == b'>')? + start;
    let mut index = end + 1;
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        index += 1;
    }
    if bytes.get(index) == Some(&b')') {
        return Some(index);
    }
    let title_start = index;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            b')' => {
                return (index > title_start).then_some(index);
            }
            _ => index += 1,
        }
    }
    None
}
