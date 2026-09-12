use super::common::{blank, blank_range, repeated, starts_with, utf8_width};

pub(crate) fn swift_code_mask(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut masked = bytes.to_vec();
    mask_swift_code(bytes, &mut masked, 0, false);
    String::from_utf8(masked).expect("masking replaces UTF-8 bytes with ASCII spaces")
}

fn mask_swift_code(source: &[u8], masked: &mut [u8], mut i: usize, close_paren: bool) -> usize {
    let mut paren_depth = 0;
    while i < source.len() {
        if close_paren && source[i] == b')' {
            if paren_depth == 0 {
                // Keep the closer so `print` inside interpolation cannot join a later `(`.
                return i + 1;
            }
            paren_depth -= 1;
            i += 1;
            continue;
        }
        if close_paren && source[i] == b'(' {
            paren_depth += 1;
            i += 1;
            continue;
        }
        if starts_with(source, i, b"//") {
            i = mask_line_comment(source, masked, i);
            continue;
        }
        if starts_with(source, i, b"/*") {
            i = mask_swift_block_comment(source, masked, i);
            continue;
        }
        if let Some(literal) = swift_literal_start(source, i) {
            i = mask_swift_literal(source, masked, literal);
            continue;
        }
        i += 1;
    }
    i
}

struct SwiftLiteral {
    start: usize,
    hashes: usize,
    open: usize,
    delimiter: u8,
    delimiter_len: usize,
}

fn swift_literal_start(source: &[u8], i: usize) -> Option<SwiftLiteral> {
    if source.get(i) == Some(&b'"') {
        return Some(swift_string_literal(i, 0, i, source));
    }
    if source.get(i) != Some(&b'#') {
        return None;
    }
    let hashes = source[i..].iter().take_while(|&&byte| byte == b'#').count();
    let open = i + hashes;
    match source.get(open) {
        Some(&b'"') => Some(swift_string_literal(i, hashes, open, source)),
        Some(&b'/') => Some(SwiftLiteral {
            start: i,
            hashes,
            open,
            delimiter: b'/',
            delimiter_len: 1,
        }),
        _ => None,
    }
}

fn swift_string_literal(start: usize, hashes: usize, open: usize, source: &[u8]) -> SwiftLiteral {
    SwiftLiteral {
        start,
        hashes,
        open,
        delimiter: b'"',
        delimiter_len: if starts_with(source, open, b"\"\"\"") {
            3
        } else {
            1
        },
    }
}

fn mask_swift_literal(source: &[u8], masked: &mut [u8], literal: SwiftLiteral) -> usize {
    let mut i = literal.open + literal.delimiter_len;
    blank_range(masked, literal.start, i);
    while i < source.len() {
        if source[i] == b'\\' {
            if interpolation_starts(source, i, literal.hashes) {
                let open_end = i + 2 + literal.hashes;
                blank_range(masked, i, open_end);
                i = mask_swift_code(source, masked, open_end, true);
                continue;
            }
            if let Some(end) = swift_escape_end(source, i, literal.hashes) {
                blank_range(masked, i, end);
                i = end;
                continue;
            }
        }
        if swift_delimited_ends(
            source,
            i,
            literal.hashes,
            literal.delimiter,
            literal.delimiter_len,
        ) {
            let end = i + literal.delimiter_len + literal.hashes;
            blank_range(masked, i, end);
            return end;
        }
        blank(masked, i);
        i += 1;
    }
    i
}

fn swift_delimited_ends(
    source: &[u8],
    i: usize,
    hashes: usize,
    delimiter: u8,
    delimiter_len: usize,
) -> bool {
    repeated(source, i, delimiter, delimiter_len)
        && source
            .get(i + delimiter_len..i + delimiter_len + hashes)
            .is_some_and(|suffix| suffix.iter().all(|&byte| byte == b'#'))
}

fn interpolation_starts(source: &[u8], i: usize, hashes: usize) -> bool {
    source.get(i + 1..i + 1 + hashes).is_some_and(|suffix| {
        suffix.iter().all(|&byte| byte == b'#') && source.get(i + 1 + hashes) == Some(&b'(')
    })
}

fn swift_escape_end(source: &[u8], i: usize, hashes: usize) -> Option<usize> {
    if !repeated(source, i + 1, b'#', hashes) {
        return None;
    }
    let char_at = i + 1 + hashes;
    if char_at >= source.len() {
        return Some(char_at);
    }
    Some((char_at + utf8_width(source[char_at])).min(source.len()))
}

fn mask_swift_block_comment(source: &[u8], masked: &mut [u8], mut i: usize) -> usize {
    let mut depth = 0;
    while i < source.len() {
        if starts_with(source, i, b"/*") {
            blank_range(masked, i, i + 2);
            depth += 1;
            i += 2;
            continue;
        }
        if starts_with(source, i, b"*/") {
            blank_range(masked, i, i + 2);
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
            continue;
        }
        blank(masked, i);
        i += 1;
    }
    i
}

fn mask_line_comment(source: &[u8], masked: &mut [u8], mut i: usize) -> usize {
    while i < source.len() && source[i] != b'\n' {
        blank(masked, i);
        i += 1;
    }
    i
}
