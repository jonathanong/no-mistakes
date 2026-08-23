pub(super) fn is_identifier_start(character: char) -> bool {
    unicode_id_start::is_id_start(character) || matches!(character, '_' | '$')
}

pub(super) fn identifier_end(line: &[u8], start: usize) -> Option<usize> {
    let identifier = std::str::from_utf8(&line[start..]).ok()?;
    let mut characters = identifier.char_indices();
    let (_, first) = characters.next()?;
    if !is_identifier_start(first) {
        return None;
    }

    let mut end = first.len_utf8();
    for (index, character) in characters {
        if !is_identifier_continue(character) {
            break;
        }
        end = index + character.len_utf8();
    }
    Some(start + end)
}

fn is_identifier_continue(character: char) -> bool {
    unicode_id_start::is_id_continue(character)
        || matches!(character, '$' | '\u{200c}' | '\u{200d}')
}

pub(super) fn token_end(line: &[u8], start: usize) -> usize {
    line[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
        .count()
        + start
}

pub(super) fn keyword_allows_regex(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"return"
            | b"throw"
            | b"case"
            | b"delete"
            | b"void"
            | b"typeof"
            | b"new"
            | b"in"
            | b"instanceof"
            | b"yield"
            | b"await"
            | b"else"
            | b"do"
    )
}

pub(super) fn allows_following_regex(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b'['
            | b','
            | b':'
            | b';'
            | b'?'
            | b'!'
            | b'='
            | b'+'
            | b'-'
            | b'*'
            | b'%'
            | b'&'
            | b'|'
            | b'^'
            | b'~'
            | b'<'
            | b'>'
    )
}
