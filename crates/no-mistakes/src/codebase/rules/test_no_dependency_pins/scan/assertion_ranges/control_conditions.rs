pub(super) fn follows_control_condition(
    bytes: &[u8],
    open_paren: usize,
    non_code_ranges: &[(usize, usize)],
) -> bool {
    let end = super::skip_trivia(bytes, open_paren, non_code_ranges);
    if [b"if".as_slice(), b"while", b"for", b"with"]
        .iter()
        .any(|token| standalone_token_ends_at(bytes, end, token, non_code_ranges))
    {
        return true;
    }
    let Some(await_start) = end.checked_sub(b"await".len()) else {
        return false;
    };
    standalone_token_ends_at(bytes, end, b"await", non_code_ranges)
        && standalone_token_ends_at(
            bytes,
            super::skip_trivia(bytes, await_start, non_code_ranges),
            b"for",
            non_code_ranges,
        )
}

fn standalone_token_ends_at(
    bytes: &[u8],
    end: usize,
    token: &[u8],
    non_code_ranges: &[(usize, usize)],
) -> bool {
    let Some(start) = end.checked_sub(token.len()) else {
        return false;
    };
    if &bytes[start..end] != token
        || (start > 0
            && (!bytes[start - 1].is_ascii()
                || bytes[start - 1].is_ascii_alphanumeric()
                || matches!(bytes[start - 1], b'_' | b'$' | b'.' | b'#')))
    {
        return false;
    }
    let before = super::skip_trivia(bytes, start, non_code_ranges);
    before == 0 || !matches!(bytes[before - 1], b'.' | b'#')
}
