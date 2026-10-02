/// Skip a dollar quote whose tag follows unquoted-identifier rules.
///
/// Tags may contain Unicode letters. A `$` after an identifier character is
/// not a quote, and `$1` stays a placeholder.
pub(super) fn skip(text: &str, index: usize, line: &mut usize) -> usize {
    let bytes = text.as_bytes();
    if !text.is_char_boundary(index) {
        return index + 1;
    }
    if text[..index]
        .chars()
        .next_back()
        .is_some_and(|ch| ch == '$' || ch == '_' || ch.is_alphanumeric())
    {
        return index + 1;
    }
    let Some(tag_len) = opening_tag_len(&text[index..]) else {
        return index + 1;
    };
    let tag = &bytes[index..index + tag_len];
    let mut cursor = index + tag_len;
    while cursor + tag.len() <= bytes.len() {
        if bytes[cursor] == b'\n' {
            *line += 1;
        }
        if bytes[cursor..].starts_with(tag) {
            return cursor + tag.len();
        }
        cursor += 1;
    }
    bytes.len()
}

fn opening_tag_len(text: &str) -> Option<usize> {
    let mut chars = text.char_indices();
    if chars.next().map(|(_, ch)| ch) != Some('$') {
        return None;
    }
    let (second_at, second) = chars.next()?;
    if second == '$' {
        return Some(2);
    }
    if second != '_' && !second.is_alphabetic() {
        return None;
    }
    let mut end = second_at + second.len_utf8();
    for (_, ch) in chars {
        if ch == '$' {
            return Some(end + 1);
        }
        if ch != '_' && !ch.is_alphanumeric() {
            return None;
        }
        end += ch.len_utf8();
    }
    None
}
