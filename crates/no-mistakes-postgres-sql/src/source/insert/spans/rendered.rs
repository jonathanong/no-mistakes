//! Quote-aware comparison of an expression's source bytes with rendered SQL.
//!
//! `--` and `/* */` outside quotes are trivia. The same characters inside
//! `'...'`, `"..."`, `E'...'`, or `$tag$...$tag$` are text and must not reject
//! an otherwise exact span.

pub(super) fn source_matches_rendered(source: &str, rendered: &str) -> bool {
    let Some(without_comments) = without_comments(source) else {
        return false;
    };
    eq_ignoring_whitespace(&without_comments, rendered)
}

fn eq_ignoring_whitespace(source: &str, rendered: &str) -> bool {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .eq(rendered
            .chars()
            .filter(|character| !character.is_whitespace()))
}

/// `None` when a block comment is still open. An unclosed quote is kept: its
/// `--` or `/*` is text, and the rendered comparison decides equality.
fn without_comments(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        if matches!(bytes[index], b'\'' | b'"') {
            let quote = bytes[index];
            let end = quoted_end(
                bytes,
                index,
                quote,
                quote == b'\'' && escape_string(source, index),
            );
            out.push_str(&source[index..end]);
            index = end;
            continue;
        }
        if bytes[index] == b'$' && word_boundary(source, index) {
            match dollar_end(source, index) {
                DollarEnd::Closed(end) => {
                    out.push_str(&source[index..end]);
                    index = end;
                    continue;
                }
                DollarEnd::Unclosed => {
                    out.push_str(&source[index..]);
                    break;
                }
                DollarEnd::NotOpen => {}
            }
        }
        if starts_with(bytes, index, b"/*") {
            index = block_comment_end(bytes, index)?;
            continue;
        }
        if starts_with(bytes, index, b"--") {
            index = line_comment_end(bytes, index);
            continue;
        }
        let width = utf8_width(bytes[index]);
        out.push_str(&source[index..index + width]);
        index += width;
    }
    Some(out)
}

/// Lead-byte width. The source is valid UTF-8 and `index` stays on a boundary.
fn utf8_width(byte: u8) -> usize {
    if byte < 0x80 {
        1
    } else if byte < 0xE0 {
        2
    } else if byte < 0xF0 {
        3
    } else {
        4
    }
}

fn quoted_end(bytes: &[u8], start: usize, quote: u8, escape: bool) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        if escape && bytes[index] == b'\\' {
            index += 1;
            if index < bytes.len() {
                index += 1;
            }
            continue;
        }
        if bytes[index] == quote {
            return index + 1;
        }
        index += 1;
    }
    bytes.len()
}

/// `E'...'` treats a backslash as an escape, so `\'` does not close the literal.
fn escape_string(source: &str, quote_at: usize) -> bool {
    let Some(prefix) = source[..quote_at].chars().next_back() else {
        return false;
    };
    if !matches!(prefix, 'e' | 'E') {
        return false;
    }
    word_boundary(source, quote_at - prefix.len_utf8())
}

fn word_boundary(source: &str, index: usize) -> bool {
    let Some(previous) = source[..index].chars().next_back() else {
        return true;
    };
    !(previous == '_' || previous.is_ascii_alphanumeric() || previous.is_alphabetic())
}

enum DollarEnd {
    NotOpen,
    Closed(usize),
    Unclosed,
}

fn dollar_end(source: &str, start: usize) -> DollarEnd {
    let bytes = source.as_bytes();
    let mut index = start + 1;
    // sqlparser 0.63 tags are Unicode alphanumeric or `_`. `$` closes the tag.
    for character in source[index..].chars() {
        if !is_dollar_tag_char(character) {
            break;
        }
        index += character.len_utf8();
    }
    if bytes.get(index) != Some(&b'$') {
        return DollarEnd::NotOpen;
    }
    let delimiter = &bytes[start..=index];
    let body = index + 1;
    match bytes[body..]
        .windows(delimiter.len())
        .position(|window| window == delimiter)
    {
        Some(found) => DollarEnd::Closed(body + found + delimiter.len()),
        None => DollarEnd::Unclosed,
    }
}

/// Dollar-tag character. `$`, punctuation, quotes, and `-` are not tags.
fn is_dollar_tag_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn block_comment_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1;
    let mut index = start + 2;
    while index < bytes.len() {
        if starts_with(bytes, index, b"/*") {
            depth += 1;
            index += 2;
        } else if starts_with(bytes, index, b"*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Some(index);
            }
        } else {
            index += 1;
        }
    }
    None
}

fn line_comment_end(bytes: &[u8], start: usize) -> usize {
    match bytes[start..].iter().position(|byte| *byte == b'\n') {
        Some(offset) => start + offset,
        None => bytes.len(),
    }
}

fn starts_with(bytes: &[u8], index: usize, prefix: &[u8]) -> bool {
    bytes.get(index..index + prefix.len()) == Some(prefix)
}
