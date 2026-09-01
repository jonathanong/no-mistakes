use super::assertion_ranges::is_code;
use regex::Match;

pub(super) fn is_direct_argument(
    content: &str,
    matched: Match<'_>,
    literal_range: (usize, usize),
    non_code_ranges: &[(usize, usize)],
) -> bool {
    let bytes = content.as_bytes();
    let Some(open) = (matched.start()..literal_range.0)
        .rev()
        .find(|&index| bytes[index] == b'(' && is_code(non_code_ranges, index))
    else {
        return false;
    };
    if skip_same_line_trivia(bytes, open + 1, non_code_ranges) != Some(literal_range.0) {
        return false;
    }
    let Some(mut end) = skip_same_line_trivia(bytes, literal_range.1, non_code_ranges) else {
        return false;
    };
    if bytes.get(end) == Some(&b',') {
        let Some(next) = skip_same_line_trivia(bytes, end + 1, non_code_ranges) else {
            return false;
        };
        end = next;
    }
    bytes.get(end) == Some(&b')')
}

fn skip_same_line_trivia(
    bytes: &[u8],
    mut index: usize,
    non_code_ranges: &[(usize, usize)],
) -> Option<usize> {
    loop {
        while bytes
            .get(index)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\r'))
        {
            index += 1;
        }
        let Some(range) = non_code_ranges.iter().find(|(start, _)| *start == index) else {
            return Some(index);
        };
        if !bytes[index..range.1].starts_with(b"/*") {
            return Some(index);
        }
        if bytes[index..range.1].contains(&b'\n') {
            return None;
        }
        index = range.1;
    }
}
