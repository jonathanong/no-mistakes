/// Resolve an OFFSET fact to the keyword in `sql`.
///
/// Spanless facts (`line == 0`) keep the Nth outer keyword. A parsed value
/// span keeps the keyword only when separators alone sit between them, so a
/// recovered inner span cannot attach to an earlier clause. Dollar quotes hide
/// the keyword from the tokenizer; the value's own text is checked next.
pub(super) fn resolve(
    sql: &str,
    keywords: &[(usize, usize)],
    ordinal: usize,
    line: usize,
    column: usize,
) -> Option<(usize, usize)> {
    if line == 0 {
        return keywords.get(ordinal).copied();
    }
    separator_keyword(sql, keywords, line, column).or_else(|| touching_keyword(sql, line, column))
}

fn separator_keyword(
    sql: &str,
    keywords: &[(usize, usize)],
    line: usize,
    column: usize,
) -> Option<(usize, usize)> {
    let keyword = keywords
        .partition_point(|location| *location <= (line, column))
        .checked_sub(1)
        .map(|index| keywords[index])?;
    let start = index_at(sql, keyword.0, keyword.1.saturating_add(6))?;
    let end = index_at(sql, line, column)?;
    (start <= end && only_separators(&sql[start..end])).then_some(keyword)
}

fn touching_keyword(sql: &str, line: usize, column: usize) -> Option<(usize, usize)> {
    let mut index = index_at(sql, line, column)?;
    while let Some((start, ch)) = prev_char(sql, index) {
        if !ch.is_whitespace() {
            break;
        }
        index = start;
    }
    let end = index;
    let mut start = index;
    while let Some((prev, ch)) = prev_char(sql, start) {
        if !is_word_char(ch) {
            break;
        }
        start = prev;
    }
    if start == end || !sql[start..end].eq_ignore_ascii_case("offset") {
        return None;
    }
    line_col_at(sql, start)
}

fn only_separators(mut text: &str) -> bool {
    while !text.is_empty() {
        let ch = text.chars().next().unwrap_or('\0');
        if ch.is_whitespace() {
            text = &text[ch.len_utf8()..];
            continue;
        }
        if let Some(rest) = text.strip_prefix("--") {
            let Some(newline) = rest.find('\n') else {
                return false;
            };
            text = &rest[newline + 1..];
            continue;
        }
        if let Some(rest) = text.strip_prefix("/*") {
            let Some(end) = block_comment_end(rest) else {
                return false;
            };
            text = &rest[end..];
            continue;
        }
        return false;
    }
    true
}

fn block_comment_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1usize;
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        if bytes[index] == b'/' && bytes[index + 1] == b'*' {
            depth += 1;
            index += 2;
            continue;
        }
        if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Some(index);
            }
            continue;
        }
        index += 1;
    }
    None
}

fn index_at(sql: &str, line: usize, column: usize) -> Option<usize> {
    if line == 0 || column == 0 {
        return None;
    }
    let mut current_line = 1usize;
    let mut current_column = 1usize;
    for (index, ch) in sql.char_indices() {
        if current_line == line && current_column == column {
            return Some(index);
        }
        if ch == '\n' {
            if current_line == line {
                return None;
            }
            current_line += 1;
            current_column = 1;
        } else {
            current_column += 1;
        }
    }
    (current_line == line && current_column == column).then_some(sql.len())
}

fn line_col_at(sql: &str, byte: usize) -> Option<(usize, usize)> {
    let mut line = 1usize;
    let mut column = 1usize;
    for (index, ch) in sql.char_indices() {
        if index == byte {
            return Some((line, column));
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (byte == sql.len()).then_some((line, column))
}

fn prev_char(sql: &str, index: usize) -> Option<(usize, char)> {
    let ch = sql.get(..index)?.chars().next_back()?;
    Some((index - ch.len_utf8(), ch))
}

fn is_word_char(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphanumeric()
}
