use super::common::{blank, blank_range, repeated, starts_with};

mod string;
use string::{csharp_string_start, mask_csharp_char, mask_csharp_string};

pub(crate) fn csharp_code_mask(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut masked = bytes.to_vec();
    mask_csharp_code(bytes, &mut masked, 0, None);
    String::from_utf8(masked).expect("masking replaces UTF-8 bytes with ASCII spaces")
}

pub(super) fn mask_csharp_code(
    source: &[u8],
    masked: &mut [u8],
    mut i: usize,
    closing_braces: Option<usize>,
) -> usize {
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    while i < source.len() {
        if let Some(width) = closing_braces {
            if source[i] == b'}' {
                if brace_depth == 0 && repeated(source, i, b'}', width) {
                    blank_range(masked, i, i + width);
                    return i + width;
                }
                brace_depth = brace_depth.saturating_sub(1);
                i += 1;
                continue;
            }
            if source[i] == b'{' {
                brace_depth += 1;
                i += 1;
                continue;
            }
        }
        if starts_with(source, i, b"//") {
            i = mask_line_comment(source, masked, i);
            continue;
        }
        if starts_with(source, i, b"/*") {
            i = mask_block_comment(source, masked, i);
            continue;
        }
        if source[i] == b'\'' {
            i = mask_csharp_char(source, masked, i);
            continue;
        }
        if let Some(string) = csharp_string_start(source, i) {
            i = mask_csharp_string(source, masked, string);
            continue;
        }
        if let Some(width) = closing_braces {
            if source[i] == b':' && brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                return mask_interpolation_format(source, masked, i, width);
            }
            match source[i] {
                b'(' => paren_depth += 1,
                b')' => paren_depth = paren_depth.saturating_sub(1),
                b'[' => bracket_depth += 1,
                b']' => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }
        }
        i += 1;
    }
    i
}

fn mask_interpolation_format(
    source: &[u8],
    masked: &mut [u8],
    mut i: usize,
    width: usize,
) -> usize {
    while i < source.len() {
        if source[i] == b'}' && repeated(source, i, b'}', width) {
            blank_range(masked, i, i + width);
            return i + width;
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

fn mask_block_comment(source: &[u8], masked: &mut [u8], mut i: usize) -> usize {
    while i < source.len() {
        if starts_with(source, i, b"*/") {
            blank_range(masked, i, i + 2);
            return i + 2;
        }
        blank(masked, i);
        i += 1;
    }
    i
}
