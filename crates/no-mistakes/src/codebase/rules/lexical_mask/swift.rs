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
                blank(masked, i);
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
        if let Some((start, hashes, quote)) = swift_string_start(source, i) {
            i = mask_swift_string(source, masked, start, hashes, quote);
            continue;
        }
        i += 1;
    }
    i
}

fn swift_string_start(source: &[u8], i: usize) -> Option<(usize, usize, usize)> {
    if source.get(i) == Some(&b'"') {
        return Some((i, 0, i));
    }
    if source.get(i) != Some(&b'#') {
        return None;
    }
    let hashes = source[i..].iter().take_while(|&&byte| byte == b'#').count();
    let quote = i + hashes;
    (source.get(quote) == Some(&b'"')).then_some((i, hashes, quote))
}

fn mask_swift_string(
    source: &[u8],
    masked: &mut [u8],
    start: usize,
    hashes: usize,
    quote: usize,
) -> usize {
    let triple = starts_with(source, quote, b"\"\"\"");
    let delimiter_len = if triple { 3 } else { 1 };
    let mut i = quote + delimiter_len;
    blank_range(masked, start, i);
    while i < source.len() {
        if swift_string_ends(source, i, hashes, delimiter_len) {
            let end = i + delimiter_len + hashes;
            blank_range(masked, i, end);
            return end;
        }
        if source[i] == b'\\' && interpolation_starts(source, i, hashes) {
            let open_end = i + 2 + hashes;
            blank_range(masked, i, open_end);
            i = mask_swift_code(source, masked, open_end, true);
            continue;
        }
        if hashes == 0 && source[i] == b'\\' {
            blank(masked, i);
            i += 1;
            if i < source.len() {
                let width = utf8_width(source[i]);
                blank_range(masked, i, (i + width).min(source.len()));
                i += width;
            }
            continue;
        }
        blank(masked, i);
        i += 1;
    }
    i
}

fn swift_string_ends(source: &[u8], i: usize, hashes: usize, delimiter_len: usize) -> bool {
    repeated(source, i, b'"', delimiter_len)
        && source
            .get(i + delimiter_len..i + delimiter_len + hashes)
            .is_some_and(|suffix| suffix.iter().all(|&byte| byte == b'#'))
}

fn interpolation_starts(source: &[u8], i: usize, hashes: usize) -> bool {
    source.get(i + 1..i + 1 + hashes).is_some_and(|suffix| {
        suffix.iter().all(|&byte| byte == b'#') && source.get(i + 1 + hashes) == Some(&b'(')
    })
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
