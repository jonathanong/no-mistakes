pub(super) fn has_matching_version_delimiters(matched: &str, raw_assertion: bool) -> bool {
    let bytes = matched.as_bytes();
    if !raw_assertion && bytes.first().is_some_and(|byte| is_quote(*byte)) {
        return has_matching_leading_delimiters(bytes);
    }
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
    (raw_assertion || !escaped)
        && prefix
            .iter()
            .rfind(|byte| is_quote(**byte))
            .is_some_and(|opening| *opening == closing)
}

fn has_matching_leading_delimiters(bytes: &[u8]) -> bool {
    let opening = bytes[0];
    bytes[1..]
        .iter()
        .enumerate()
        .find(|(_, byte)| is_quote(**byte))
        .is_some_and(|(offset, closing)| {
            let index = offset + 1;
            let escaped = bytes[..index]
                .iter()
                .rev()
                .take_while(|byte| **byte == b'\\')
                .count()
                % 2
                == 1;
            !escaped && *closing == opening
        })
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
