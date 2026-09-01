pub(super) fn assertion_ranges(content: &str) -> Vec<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut stack = Vec::new();
    let mut ranges = Vec::new();
    let mut quote = None::<u8>;
    let mut escaped = false;
    let mut regex = false;
    let mut regex_class = false;
    let mut regex_allowed = true;
    let mut line_comment = false;
    let mut block_comment = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        if line_comment {
            line_comment = byte != b'\n';
            index += 1;
            continue;
        }
        if block_comment {
            if byte == b'*' && next == Some(b'/') {
                block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
                regex_allowed = false;
            }
            index += 1;
            continue;
        }
        if regex {
            if escaped {
                escaped = false;
            } else {
                match byte {
                    b'\\' => escaped = true,
                    b'[' => regex_class = true,
                    b']' => regex_class = false,
                    b'/' if !regex_class => {
                        regex = false;
                        regex_allowed = false;
                    }
                    b'\n' => {
                        regex = false;
                        regex_class = false;
                    }
                    _ => {}
                }
            }
            index += 1;
            continue;
        }
        match byte {
            b'/' if next == Some(b'/') => {
                line_comment = true;
                index += 2;
                continue;
            }
            b'/' if next == Some(b'*') => {
                block_comment = true;
                index += 2;
                continue;
            }
            b'/' if regex_allowed => regex = true,
            b'/' => regex_allowed = true,
            b'\'' | b'"' | b'`' => quote = Some(byte),
            b'(' => {
                stack.push(expect_token_start(bytes, index));
                regex_allowed = true;
            }
            b')' => {
                if let Some(Some(start)) = stack.pop() {
                    ranges.push((start, index));
                }
                regex_allowed = false;
            }
            b'[' | b'{' | b',' | b';' | b':' | b'=' | b'!' | b'?' | b'&' | b'|' | b'+' | b'-'
            | b'*' | b'%' | b'~' | b'^' | b'<' | b'>' => regex_allowed = true,
            b']' | b'}' | b'.' => regex_allowed = false,
            byte if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && (bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'_' | b'$'))
                {
                    index += 1;
                }
                regex_allowed = matches!(
                    &bytes[start..index],
                    b"return"
                        | b"throw"
                        | b"case"
                        | b"delete"
                        | b"void"
                        | b"typeof"
                        | b"yield"
                        | b"await"
                );
                continue;
            }
            byte if !byte.is_ascii_whitespace() => regex_allowed = false,
            _ => {}
        }
        index += 1;
    }
    ranges.extend(
        stack
            .into_iter()
            .flatten()
            .map(|start| (start, bytes.len())),
    );
    ranges.sort_unstable_by_key(|(start, _)| *start);
    ranges
}

pub(super) fn assertion_start(ranges: &[(usize, usize)], match_start: usize) -> usize {
    let upper = ranges.partition_point(|(start, _)| *start <= match_start);
    upper
        .checked_sub(1)
        .and_then(|index| {
            let (start, end) = ranges[index];
            (match_start <= end).then_some(start)
        })
        .unwrap_or(match_start)
}

pub(super) fn expect_token_start(bytes: &[u8], open_paren: usize) -> Option<usize> {
    let mut end = open_paren;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    for token in [b"expect.soft".as_slice(), b"expect.poll", b"expect"] {
        let Some(start) = end.checked_sub(token.len()) else {
            continue;
        };
        if &bytes[start..end] == token
            && (start == 0
                || !(bytes[start - 1].is_ascii_alphanumeric()
                    || matches!(bytes[start - 1], b'_' | b'$' | b'.')))
        {
            return Some(start);
        }
    }
    None
}
