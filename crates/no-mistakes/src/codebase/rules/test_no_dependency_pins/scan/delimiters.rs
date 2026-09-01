pub(super) fn has_matching_version_delimiters(matched: &str, allow_escaped: bool) -> bool {
    let bytes = matched.as_bytes();
    let Some(&closing) = bytes.last() else {
        return false;
    };
    let prefix = &bytes[..bytes.len() - 1];
    let escaped = prefix
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count()
        % 2
        == 1;
    (allow_escaped || !escaped)
        && prefix
            .iter()
            .rfind(|byte| matches!(byte, b'\'' | b'"' | b'`'))
            .is_some_and(|opening| *opening == closing)
}

pub(super) fn has_matching_raw_entry_delimiters(matched: &str) -> bool {
    let mut quotes = matched
        .bytes()
        .rev()
        .filter(|byte| matches!(byte, b'\'' | b'"' | b'`'));
    let (Some(value_close), Some(value_open), Some(key_close), Some(key_open)) =
        (quotes.next(), quotes.next(), quotes.next(), quotes.next())
    else {
        return false;
    };
    value_open == value_close && key_open == key_close
}
