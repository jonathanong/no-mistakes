pub(super) fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

pub(super) fn identifier_end(line: &[u8], start: usize) -> usize {
    line[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
        .count()
        + start
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
