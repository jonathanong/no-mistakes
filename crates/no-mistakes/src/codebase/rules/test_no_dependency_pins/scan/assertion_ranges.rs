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
            b'/' if lex.regex_allowed => {
                lex.regex = true;
                lex.enter_non_code(index);
            }
            b'/' => lex.regex_allowed = true,
            b'\'' | b'"' | b'`' => {
                lex.quote = Some(byte);
                lex.enter_non_code(index);
            }
            b'(' => {
                stack.push(expect_token_start(bytes, index));
                lex.regex_allowed = true;
            }
            b')' => {
                if let Some(Some(start)) = stack.pop() {
                    ranges.push((start, index));
                }
                lex.regex_allowed = false;
            }
            b'[' | b'{' | b',' | b';' | b':' | b'=' | b'!' | b'?' | b'&' | b'|' | b'+' | b'-'
            | b'*' | b'%' | b'~' | b'^' | b'<' | b'>' => lex.regex_allowed = true,
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
