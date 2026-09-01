mod lex_state;

use lex_state::LexState;

#[derive(Default)]
pub(super) struct SourceRanges {
    pub(super) assertions: Vec<(usize, usize)>,
    pub(super) non_code: Vec<(usize, usize)>,
}

pub(super) fn source_ranges(content: &str) -> SourceRanges {
    let bytes = content.as_bytes();
    let mut stack = Vec::new();
    let mut ranges = Vec::new();
    let mut non_code_ranges = Vec::new();
    let mut lex = LexState::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        if let Some(next_index) = lex.skip_non_code(bytes, index, &mut non_code_ranges) {
            index = next_index;
            continue;
        }
        match byte {
            b'/' if next == Some(b'/') => {
                lex.line_comment = true;
                lex.enter_non_code(index);
                index += 2;
                continue;
            }
            b'/' if next == Some(b'*') => {
                lex.block_comment = true;
                lex.enter_non_code(index);
                index += 2;
                continue;
            }
            b'/' if lex.regex_allowed && (index == 0 || bytes[index - 1] != b'<') => {
                lex.regex = true;
                lex.enter_non_code(index);
            }
            b'/' => lex.regex_allowed = true,
            b'\'' | b'"' | b'`'
                if byte == b'`'
                    || lex.regex_allowed
                    || index == 0
                    || !(bytes[index - 1].is_ascii_alphanumeric()
                        || matches!(bytes[index - 1], b'_' | b'$')) =>
            {
                lex.quote = Some(byte);
                lex.enter_non_code(index);
            }
            b'(' => {
                stack.push(expect_token_start(bytes, index, &non_code_ranges));
                lex.regex_allowed = true;
            }
            b')' => {
                if let Some(Some(start)) = stack.pop() {
                    ranges.push((start, index));
                }
                lex.regex_allowed = false;
            }
            b'{' => {
                lex.open_brace();
                lex.regex_allowed = true;
            }
            b'}' if lex.close_brace(index) => {}
            byte @ (b'+' | b'-') if next == Some(byte) => {
                index += 2;
                continue;
            }
            b'[' | b',' | b';' | b':' | b'=' | b'!' | b'?' | b'&' | b'|' | b'+' | b'-' | b'*'
            | b'%' | b'~' | b'^' | b'<' | b'>' => lex.regex_allowed = true,
            b']' | b'}' | b'.' => lex.regex_allowed = false,
            byte if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && (bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'_' | b'$'))
                {
                    index += 1;
                }
                lex.regex_allowed = matches!(
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
            byte if !byte.is_ascii_whitespace() => lex.regex_allowed = false,
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
    lex.finish_non_code(bytes.len(), &mut non_code_ranges);
    ranges.sort_unstable_by_key(|(start, _)| *start);
    SourceRanges {
        assertions: ranges,
        non_code: non_code_ranges,
    }
}

pub(super) fn is_code(ranges: &[(usize, usize)], offset: usize) -> bool {
    let upper = ranges.partition_point(|(start, _)| *start <= offset);
    upper == 0 || offset >= ranges[upper - 1].1
}

pub(super) fn assertion_start(ranges: &[(usize, usize)], match_start: usize) -> Option<usize> {
    let upper = ranges.partition_point(|(start, _)| *start <= match_start);
    ranges[..upper]
        .iter()
        .rev()
        .find_map(|&(start, end)| (match_start <= end).then_some(start))
}

pub(super) fn expect_token_start(
    bytes: &[u8],
    open_paren: usize,
    non_code_ranges: &[(usize, usize)],
) -> Option<usize> {
    let mut end = skip_trivia(bytes, open_paren, non_code_ranges);
    end -= usize::from(end > 0 && bytes[end - 1] == b'!');
    end = type_argument_start(bytes, end, non_code_ranges)?;
    end = skip_trivia(bytes, end, non_code_ranges);
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
fn skip_trivia(bytes: &[u8], mut end: usize, non_code_ranges: &[(usize, usize)]) -> usize {
    loop {
        while end > 0 && bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        let upper = non_code_ranges.partition_point(|(start, _)| *start < end);
        let Some(&(start, range_end)) = non_code_ranges[..upper].last() else {
            return end;
        };
        if range_end != end
            || !(bytes[start..].starts_with(b"//") || bytes[start..].starts_with(b"/*"))
        {
            return end;
        }
        end = start;
    }
}

fn type_argument_start(
    bytes: &[u8],
    end: usize,
    non_code_ranges: &[(usize, usize)],
) -> Option<usize> {
    if end == 0 || bytes[end - 1] != b'>' {
        return Some(end);
    }
    let mut depth = 0;
    let mut index = end;
    while index > 0 {
        index -= 1;
        let upper = non_code_ranges.partition_point(|(start, _)| *start <= index);
        if let Some(&(start, range_end)) = non_code_ranges[..upper].last() {
            if index < range_end {
                index = start;
                continue;
            }
        }
        match bytes[index] {
            b'>' if index == 0 || bytes[index - 1] != b'=' => depth += 1,
            b'<' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}
