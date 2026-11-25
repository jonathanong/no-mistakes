use super::CommentIndex;

pub(super) fn find_eol(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
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
            col += 1;
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
            if bytes.get(i + 1) == Some(&b'\n') {
                line += 1;
                col = 1;
            } else {
                col += 2;
            }
            i = (i + 2).min(bytes.len());
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
            col += 1;
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
    let mut i = end + 1;
    col += delim.len();
    while i < bytes.len() {
        if bytes[i..].starts_with(delim) {
            return (i + delim.len(), line, col + delim.len());
        }
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (bytes.len(), line, col)
}

pub(super) fn scan_js_quoted(
    bytes: &[u8],
    start: usize,
    mut line: usize,
    mut col: usize,
) -> (usize, usize, usize) {
    let quote = bytes[start];
    let mut i = start + 1;
    col += 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if bytes.get(i + 1) == Some(&b'\n') {
                line += 1;
                col = 1;
            } else {
                col += 2;
            }
            i = (i + 2).min(bytes.len());
        } else if bytes[i] == quote {
            return (i + 1, line, col + 1);
        } else if bytes[i] == b'\n' {
            line += 1;
            col = 1;
            i += 1;
        } else {
            col += 1;
            i += 1;
        }
    }
    (bytes.len(), line, col)
}

pub(super) fn scan_js_template(
    bytes: &[u8],
    mut i: usize,
    mut line: usize,
    mut col: usize,
    index: &mut CommentIndex,
    braces: &mut Vec<usize>,
) -> (usize, usize, usize) {
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if bytes.get(i + 1) == Some(&b'\n') {
                line += 1;
                col = 1;
            } else {
                col += 2;
            }
            i = (i + 2).min(bytes.len());
        } else if bytes[i] == b'`' {
            index.mark_code(line, col);
            return (i + 1, line, col + 1);
        } else if bytes.get(i..i + 2) == Some(b"${") {
            index.mark_code(line, col);
            braces.push(0);
            return (i + 2, line, col + 2);
        } else if bytes[i] == b'\n' {
            line += 1;
            col = 1;
            i += 1;
        } else {
            index.mark_code(line, col);
            col += 1;
            i += 1;
        }
    }
    (bytes.len(), line, col)
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
}
