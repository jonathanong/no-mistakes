use super::super::common::{blank, blank_range, repeated, utf8_width};
use super::mask_csharp_code;

pub(super) struct CsharpString {
    pub(super) start: usize,
    pub(super) quote: usize,
    pub(super) dollars: usize,
    pub(super) verbatim: bool,
    pub(super) quote_count: usize,
}

pub(super) fn csharp_string_start(source: &[u8], i: usize) -> Option<CsharpString> {
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

pub(super) fn mask_csharp_string(source: &[u8], masked: &mut [u8], string: CsharpString) -> usize {
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

pub(super) fn mask_csharp_char(source: &[u8], masked: &mut [u8], mut i: usize) -> usize {
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
