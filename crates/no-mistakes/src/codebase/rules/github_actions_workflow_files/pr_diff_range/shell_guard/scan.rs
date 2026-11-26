use std::ops::Range;

mod arithmetic;

pub(super) use arithmetic::has_arithmetic_in_range;

pub(super) struct HeredocSpan {
    pub(super) range: Range<usize>,
    pub(super) quoted: bool,
}

pub(super) fn has_complex_expansion_in_range(
    script: &str,
    range: Range<usize>,
    heredocs: &[HeredocSpan],
) -> bool {
    let mut cursor = range.start;
    for doc in heredocs {
        if doc.range.end <= cursor {
            continue;
        }
        if doc.range.start >= range.end {
            break;
        }
        if doc.range.start > cursor && has_complex_expansion(&script[cursor..doc.range.start]) {
            return true;
        }
        let body_start = doc.range.start.max(cursor);
        let body_end = doc.range.end.min(range.end);
        if !doc.quoted && has_unquoted_heredoc_expansion(&script[body_start..body_end]) {
            return true;
        }
        cursor = cursor.max(body_end);
    }
    cursor < range.end && has_complex_expansion(&script[cursor..range.end])
}

fn has_unquoted_heredoc_expansion(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\'
            && bytes
                .get(i + 1)
                .is_some_and(|next| matches!(next, b'$' | b'`' | b'\\' | b'\n'))
        {
            i += 2;
        } else {
            match check_expansion(bytes, i) {
                Expansion::Complex => return true,
                Expansion::Simple(next) => i = next,
                Expansion::None => i += 1,
            }
        }
    }
    false
}

fn has_complex_expansion(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut in_double = false;
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if in_double {
            if byte == b'\\'
                && bytes
                    .get(i + 1)
                    .is_some_and(|next| matches!(next, b'"' | b'\\' | b'$' | b'`' | b'\n'))
            {
                i += 2;
            } else if byte == b'"' {
                in_double = false;
                i += 1;
            } else {
                match check_expansion(bytes, i) {
                    Expansion::Complex => return true,
                    Expansion::Simple(next) => i = next,
                    Expansion::None => i += 1,
                }
            }
        } else if byte == b'\\' {
            i += 2;
        } else if byte == b'\'' {
            i = skip_single(bytes, i + 1);
        } else if byte == b'$' && bytes.get(i + 1) == Some(&b'\'') {
            i = skip_ansi_c(bytes, i + 2);
        } else if byte == b'"' {
            in_double = true;
            i += 1;
        } else if is_comment_start(bytes, i) {
            i = skip_comment(bytes, i);
        } else {
            match check_expansion(bytes, i) {
                Expansion::Complex => return true,
                Expansion::Simple(next) => i = next,
                Expansion::None => i += 1,
            }
        }
    }
    false
}

fn skip_single(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\'' {
        i += 1;
    }
    (i + 1).min(bytes.len())
}

fn skip_ansi_c(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\'' {
        i += if bytes[i] == b'\\' && i + 1 < bytes.len() {
            2
        } else {
            1
        };
    }
    (i + 1).min(bytes.len())
}

fn is_comment_start(bytes: &[u8], i: usize) -> bool {
    bytes[i] == b'#'
        && (i == 0
            || bytes[i - 1].is_ascii_whitespace()
            || matches!(bytes[i - 1], b';' | b'|' | b'&'))
}

fn skip_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

enum Expansion {
    None,
    Simple(usize),
    Complex,
}

fn check_expansion(bytes: &[u8], i: usize) -> Expansion {
    if bytes[i] == b'`' {
        return Expansion::Complex;
    }
    if bytes[i] != b'$' {
        return Expansion::None;
    }
    match bytes.get(i + 1) {
        Some(b'(' | b'[') => Expansion::Complex,
        Some(b'{') => {
            let start = i + 2;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start && bytes.get(end) == Some(&b'}') {
                Expansion::Simple(end + 1)
            } else {
                Expansion::Complex
            }
        }
        _ => Expansion::None,
    }
}
