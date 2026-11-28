pub(super) fn is_utf8_cont(b: u8) -> bool {
    (b & 0xC0) == 0x80
}

pub(super) fn find_eol(bytes: &[u8], mut i: usize) -> (usize, usize) {
    let mut chars = 0usize;
    while i < bytes.len() && bytes[i] != b'\n' {
        if !is_utf8_cont(bytes[i]) {
            chars += 1;
        }
        i += 1;
    }
    (i, chars)
}

pub(super) fn advance_utf8_char(source: &str, i: usize, col: usize) -> (usize, usize) {
    let ch = source[i..].chars().next().unwrap();
    (i + ch.len_utf8(), col + 1)
}

pub(super) fn scan_block_comment(
    bytes: &[u8],
    start: usize,
    mut line: usize,
    mut col: usize,
    nested: bool,
) -> (usize, usize, usize, usize) {
    let mut i = start + 2;
    col += 2;
    let mut depth = 1usize;
    while i < bytes.len() {
        if nested && bytes.get(i..i + 2) == Some(b"/*") {
            depth += 1;
            i += 2;
            col += 2;
        } else if bytes.get(i..i + 2) == Some(b"*/") {
            depth -= 1;
            if depth == 0 {
                return (i, i + 2, line, col + 2);
            }
            i += 2;
            col += 2;
        } else if bytes[i] == b'\n' {
            line += 1;
            col = 1;
            i += 1;
        } else {
            if !is_utf8_cont(bytes[i]) {
                col += 1;
            }
            i += 1;
        }
    }
    (bytes.len(), bytes.len(), line, col)
}

pub(super) fn scan_sql_quoted(
    bytes: &[u8],
    start: usize,
    mut line: usize,
    mut col: usize,
) -> (usize, usize, usize) {
    let quote = bytes[start];
    let escape = quote == b'\''
        && start > 0
        && matches!(bytes[start - 1], b'e' | b'E')
        && (start == 1 || !is_ident(bytes[start - 2]));
    let mut i = start + 1;
    col += 1;
    while i < bytes.len() {
        if escape && bytes[i] == b'\\' {
            i = scan_escaped(bytes, i, &mut line, &mut col);
        } else if bytes[i] == quote {
            i += 1;
            col += 1;
            if bytes.get(i) == Some(&quote) {
                i += 1;
                col += 1;
            } else {
                break;
            }
        } else if bytes[i] == b'\n' {
            line += 1;
            col = 1;
            i += 1;
        } else {
            if !is_utf8_cont(bytes[i]) {
                col += 1;
            }
            i += 1;
        }
    }
    (i, line, col)
}

pub(super) fn scan_dollar_quote_or_char(
    bytes: &[u8],
    start: usize,
    mut line: usize,
    mut col: usize,
) -> (usize, usize, usize) {
    if start > 0 && is_ident(bytes[start - 1]) {
        return (start + 1, line, col + 1);
    }
    let mut end = start + 1;
    if bytes.get(end).is_some_and(u8::is_ascii_digit) {
        return (start + 1, line, col + 1);
    }
    while bytes.get(end).is_some_and(|&b| is_ident(b)) {
        end += 1;
    }
    if bytes.get(end) != Some(&b'$') {
        return (start + 1, line, col + 1);
    }
    let delim = &bytes[start..=end];
    let delim_chars = delim.iter().filter(|&&b| !is_utf8_cont(b)).count();
    let mut i = end + 1;
    col += delim_chars;
    while i < bytes.len() {
        if bytes[i..].starts_with(delim) {
            return (i + delim.len(), line, col + delim_chars);
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else if !is_utf8_cont(bytes[i]) {
            col += 1;
        }
        i += 1;
    }
    (bytes.len(), line, col)
}

pub(super) fn scan_escaped(bytes: &[u8], i: usize, line: &mut usize, col: &mut usize) -> usize {
    if bytes.get(i + 1) == Some(&b'\n') {
        *line += 1;
        *col = 1;
    } else {
        *col += 2;
    }
    (i + 2).min(bytes.len())
}

pub(super) fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
}
