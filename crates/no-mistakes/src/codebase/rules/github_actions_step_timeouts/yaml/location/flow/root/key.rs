pub(super) fn mapping_key_len(text: &str, wanted: &str) -> Option<usize> {
    if text
        .strip_prefix(wanted)
        .is_some_and(|rest| rest.starts_with(':'))
    {
        return Some(wanted.len() + 1);
    }
    for quote in ['"', '\''] {
        if text.starts_with(quote)
            && text[quote.len_utf8()..]
                .strip_prefix(wanted)
                .is_some_and(|rest| {
                    rest.starts_with(quote) && rest[quote.len_utf8()..].starts_with(':')
                })
        {
            return Some(wanted.len() + 3);
        }
    }
    None
}

pub(super) fn key_boundary(previous: Option<char>) -> bool {
    previous.is_none_or(|char| char.is_whitespace() || char == '{' || char == ',')
}
