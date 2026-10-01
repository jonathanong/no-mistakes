/// Byte offset of `words` in `sql`, skipping comments and quoted text.
///
/// The parser drops token locations, so declaration lines come from this scan.
pub(super) fn find_opening(sql: &str, words: &[&str]) -> Option<(usize, usize)> {
    let lower = sql.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'-' && bytes.get(index + 1) == Some(&b'-') {
            index = skip_line(&lower, index);
            continue;
        }
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index = skip_block(&lower, index);
            continue;
        }
        if bytes[index] == b'\'' || bytes[index] == b'"' {
            index = skip_quoted(&lower, index);
            continue;
        }
        if let Some(end) = match_words(&lower, index, words) {
            return Some((index, end));
        }
        index += lower[index..].chars().next().map_or(1, char::len_utf8);
    }
    None
}

pub(super) fn line_number(sql: &str, byte: usize) -> usize {
    sql.get(..byte)
        .unwrap_or(sql)
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn match_words(sql: &str, start: usize, words: &[&str]) -> Option<usize> {
    let mut at = start;
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            at = skip_whitespace(sql, at)?;
        }
        if !sql[at..].starts_with(word) || !word_boundary(sql, at + word.len()) {
            return None;
        }
        at += word.len();
    }
    Some(at)
}

fn word_boundary(sql: &str, at: usize) -> bool {
    sql[at..].chars().next().is_none_or(|character| {
        !(character.is_alphanumeric() || character == '_' || character == '$')
    })
}

fn skip_whitespace(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut at = start;
    while at < bytes.len() && bytes[at].is_ascii_whitespace() {
        at += 1;
    }
    (at > start).then_some(at)
}

fn skip_line(sql: &str, start: usize) -> usize {
    sql[start..]
        .find('\n')
        .map_or(sql.len(), |offset| start + offset + 1)
}

fn skip_block(sql: &str, start: usize) -> usize {
    sql[start + 2..]
        .find("*/")
        .map_or(sql.len(), |offset| start + 2 + offset + 2)
}

fn skip_quoted(sql: &str, start: usize) -> usize {
    let bytes = sql.as_bytes();
    let quote = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == quote {
            if bytes.get(index + 1) == Some(&quote) {
                index += 2;
                continue;
            }
            return index + 1;
        }
        index += 1;
    }
    sql.len()
}
