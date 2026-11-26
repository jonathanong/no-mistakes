use super::{is_comment_start, skip_ansi_c, skip_comment, skip_single};
use std::ops::Range;

pub(in super::super) fn has_arithmetic_in_range(
    script: &str,
    range: Range<usize>,
    heredocs: &[Range<usize>],
) -> bool {
    let mut cursor = range.start;
    for doc in heredocs {
        if doc.end <= cursor {
            continue;
        }
        if doc.start >= range.end {
            break;
        }
        if doc.start > cursor && has_arithmetic_command(&script[cursor..doc.start]) {
            return true;
        }
        cursor = cursor.max(doc.end.min(range.end));
    }
    cursor < range.end && has_arithmetic_command(&script[cursor..range.end])
}

fn has_arithmetic_command(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut in_double = false;
    let mut at_cmd_start = true;
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if in_double {
            if byte == b'\\' && i + 1 < bytes.len() {
                i += 2;
            } else {
                in_double = byte != b'"';
                i += 1;
            }
        } else if byte == b'\\' {
            at_cmd_start = false;
            i += 2;
        } else if byte == b'\'' {
            at_cmd_start = false;
            i = skip_single(bytes, i + 1);
        } else if byte == b'$' && bytes.get(i + 1) == Some(&b'\'') {
            at_cmd_start = false;
            i = skip_ansi_c(bytes, i + 2);
        } else if byte == b'"' {
            at_cmd_start = false;
            in_double = true;
            i += 1;
        } else if is_comment_start(bytes, i) {
            i = skip_comment(bytes, i);
        } else if matches!(byte, b';' | b'\n' | b'|' | b'&') {
            at_cmd_start = true;
            i += 1;
        } else if byte.is_ascii_whitespace() {
            i += 1;
        } else if at_cmd_start && byte == b'(' && bytes.get(i + 1) == Some(&b'(') {
            return true;
        } else {
            at_cmd_start = false;
            i += 1;
        }
    }
    false
}
