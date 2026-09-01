pub(super) fn has_matching_version_delimiters(matched: &str, raw_assertion: bool) -> bool {
    let bytes = matched.as_bytes();
    if raw_assertion {
        return has_matching_trailing_delimiters(bytes, true);
    }
    let (Some(&opening), Some(&closing)) = (bytes.first(), bytes.last()) else {
        return false;
    };
    opening == closing && is_quote(opening) && !is_escaped(&bytes[..bytes.len() - 1])
}

fn has_matching_trailing_delimiters(bytes: &[u8], raw_assertion: bool) -> bool {
    let Some(&closing) = bytes.last() else {
        return false;
    };
    let prefix = &bytes[..bytes.len() - 1];
    (raw_assertion || !is_escaped(prefix))
        && prefix
            .iter()
            .rfind(|byte| is_quote(**byte))
            .is_some_and(|opening| *opening == closing)
}

fn is_escaped(prefix: &[u8]) -> bool {
    let escaped = prefix
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count()
        % 2
        == 1;
    escaped
}

pub(super) fn has_matching_raw_entry_delimiters(matched: &str) -> bool {
    let mut quotes = matched.bytes().rev().filter(|byte| is_quote(*byte));
    let (Some(value_close), Some(value_open), Some(key_close), Some(key_open)) =
        (quotes.next(), quotes.next(), quotes.next(), quotes.next())
    else {
        return false;
    };
    value_open == value_close && key_open == key_close
}

fn is_quote(byte: u8) -> bool {
    matches!(byte, b'\'' | b'"' | b'`')
}
