use super::common::{blank, blank_range, repeated, starts_with, utf8_width};

pub(crate) fn csharp_code_mask(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut masked = bytes.to_vec();
    mask_csharp_code(bytes, &mut masked, 0, None);
    String::from_utf8(masked).expect("masking replaces UTF-8 bytes with ASCII spaces")
}

fn mask_csharp_code(
    source: &[u8],
    masked: &mut [u8],
    mut i: usize,
    closing_braces: Option<usize>,
) -> usize {
    let mut brace_depth = 0;
    while i < source.len() {
        if let Some(width) = closing_braces {
            if source[i] == b'}' {
                if brace_depth == 0 && repeated(source, i, b'}', width) {
                    blank_range(masked, i, i + width);
                    return i + width;
                }
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
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
        i += 1;
    }
    i
}

#[derive(Clone, Copy)]
struct CsharpString {
    start: usize,
    quote: usize,
    dollars: usize,
    verbatim: bool,
    quote_count: usize,
}

fn csharp_string_start(source: &[u8], i: usize) -> Option<CsharpString> {
    let mut j = i;
    let mut dollars = 0;
    while source.get(j) == Some(&b'$') {
        dollars += 1;
        j += 1;
    }
    let mut verbatim = false;
    if source.get(j) == Some(&b'@') {
        verbatim = true;
        j += 1;
        while source.get(j) == Some(&b'$') {
            dollars += 1;
            j += 1;
        }
    }
    if source.get(j) != Some(&b'"') {
        return None;
    }
    let quote_count = source[j..].iter().take_while(|&&byte| byte == b'"').count();
    Some(CsharpString {
        start: i,
        quote: j,
        dollars,
        verbatim,
        quote_count,
    })
}

fn mask_csharp_string(source: &[u8], masked: &mut [u8], string: CsharpString) -> usize {
    let raw = string.quote_count >= 3;
    let delimiter_len = if raw { string.quote_count } else { 1 };
    let mut i = string.quote + delimiter_len;
    blank_range(masked, string.start, i);
    while i < source.len() {
        if raw && repeated(source, i, b'"', delimiter_len) {
            blank_range(masked, i, i + delimiter_len);
            return i + delimiter_len;
        }
        if !raw && source[i] == b'"' {
            if string.verbatim && source.get(i + 1) == Some(&b'"') {
                blank_range(masked, i, i + 2);
                i += 2;
                continue;
            }
            blank(masked, i);
            return i + 1;
        }
        if !raw
            && string.dollars > 0
            && csharp_escaped_interpolation_braces(source, i, string.dollars, source[i])
        {
            let escaped_width = string.dollars * 2;
            blank_range(masked, i, i + escaped_width);
            i += escaped_width;
            continue;
        }
        if string.dollars > 0 && csharp_interpolation_starts(source, i, string.dollars) {
            blank_range(masked, i, i + string.dollars);
            i = mask_csharp_code(source, masked, i + string.dollars, Some(string.dollars));
            continue;
        }
        if !raw && !string.verbatim && source[i] == b'\\' {
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

fn csharp_interpolation_starts(source: &[u8], i: usize, width: usize) -> bool {
    repeated(source, i, b'{', width) && source.get(i + width) != Some(&b'{')
}

fn csharp_escaped_interpolation_braces(source: &[u8], i: usize, width: usize, brace: u8) -> bool {
    matches!(brace, b'{' | b'}') && repeated(source, i, brace, width * 2)
}

fn mask_csharp_char(source: &[u8], masked: &mut [u8], mut i: usize) -> usize {
    blank(masked, i);
    i += 1;
    while i < source.len() {
        if source[i] == b'\\' {
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
        if source[i] == b'\'' {
            return i + 1;
        }
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
