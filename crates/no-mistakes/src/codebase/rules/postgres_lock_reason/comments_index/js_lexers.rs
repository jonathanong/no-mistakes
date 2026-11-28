use super::lexers::{is_utf8_cont, scan_escaped};
use super::CommentIndex;

/// End of a regex literal at `start` when the previous token makes `/` a regex opener.
pub(super) fn regex_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let prev = bytes[..start]
        .iter()
        .rev()
        .find(|byte| !byte.is_ascii_whitespace());
    if prev.is_some_and(|byte| !b"(,=:[!&|?{};+-*%<>~".contains(byte)) {
        return None;
    }
    let (mut i, mut in_class) = (start + 1, false);
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'\n' => return None,
            b'[' => in_class = true,
            b']' => in_class = false,
            b'/' if !in_class => return Some(i + 1),
            _ => {}
        }
        i += 1;
    }
    None
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
            i = scan_escaped(bytes, i, &mut line, &mut col);
        } else if bytes[i] == quote {
            return (i + 1, line, col + 1);
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
            i = scan_escaped(bytes, i, &mut line, &mut col);
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
            if !is_utf8_cont(bytes[i]) {
                col += 1;
            }
            i += 1;
        }
    }
    (bytes.len(), line, col)
}

pub(super) fn scan_js_regex(
    bytes: &[u8],
    start: usize,
    mut line: usize,
    mut col: usize,
) -> (usize, usize, usize) {
    let mut i = start + 1;
    col += 1;
    let mut in_class = false;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i = scan_escaped(bytes, i, &mut line, &mut col),
            b'[' => {
                in_class = true;
                i += 1;
                col += 1;
            }
            b']' if in_class => {
                in_class = false;
                i += 1;
                col += 1;
            }
            b'/' if !in_class => {
                i += 1;
                col += 1;
                while bytes.get(i).is_some_and(u8::is_ascii_alphabetic) {
                    i += 1;
                    col += 1;
                }
                return (i, line, col);
            }
            b'\n' => return (i, line, col),
            b => {
                if !is_utf8_cont(b) {
                    col += 1;
                }
                i += 1;
            }
        }
    }
    (bytes.len(), line, col)
}

pub(super) fn regex_can_follow_word(word: &str) -> bool {
    matches!(
        word,
        "return"
            | "throw"
            | "case"
            | "in"
            | "of"
            | "instanceof"
            | "typeof"
            | "void"
            | "delete"
            | "await"
            | "yield"
            | "new"
            | "else"
            | "do"
    )
}
