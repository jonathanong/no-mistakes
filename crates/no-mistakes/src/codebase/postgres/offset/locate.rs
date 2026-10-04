/// Resolve an OFFSET fact to the keyword in `sql`.
///
/// Spanless facts (`line == 0`) keep the Nth outer keyword. A parsed value
/// span keeps the keyword only when separators alone sit between them, so a
/// recovered inner span cannot attach to an earlier clause. Dollar quotes hide
/// the keyword from the tokenizer; the value's own text is checked next.
pub(super) fn resolve(
    positions: &Positions<'_>,
    keywords: &[(usize, usize)],
    ordinal: usize,
    line: usize,
    column: usize,
) -> Option<(usize, usize)> {
    if line == 0 {
        return keywords.get(ordinal).copied();
    }
    separator_keyword(positions, keywords, line, column)
        .or_else(|| touching_keyword(positions, line, column))
}

fn separator_keyword(
    positions: &Positions<'_>,
    keywords: &[(usize, usize)],
    line: usize,
    column: usize,
) -> Option<(usize, usize)> {
    let sql = positions.sql;
    let keyword = keywords
        .partition_point(|location| *location <= (line, column))
        .checked_sub(1)
        .map(|index| keywords[index])?;
    let start = positions.index_at(keyword.0, keyword.1.saturating_add(6))?;
    let end = positions.index_at(line, column)?;
    (start <= end && only_separators(&sql[start..end])).then_some(keyword)
}

fn touching_keyword(
    positions: &Positions<'_>,
    line: usize,
    column: usize,
) -> Option<(usize, usize)> {
    let sql = positions.sql;
    let mut index = positions.index_at(line, column)?;
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
    positions.line_col_at(start)
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

pub(super) struct Positions<'a> {
    sql: &'a str,
    // (byte offset, character offset, Unicode correction count) at line starts.
    lines: Vec<(usize, usize, usize)>,
    // End offsets let byte minus character encode cumulative UTF-8 excess.
    unicode: Vec<(usize, usize)>,
}

impl<'a> Positions<'a> {
    pub(super) fn new(sql: &'a str) -> Self {
        let mut lines = vec![(0, 0, 0)];
        let mut unicode = Vec::new();
        for (character, (byte, ch)) in sql.char_indices().enumerate() {
            if ch == '\n' {
                lines.push((byte + 1, character + 1, unicode.len()));
            } else if !ch.is_ascii() {
                unicode.push((byte + ch.len_utf8(), character + 1));
            }
        }
        Self {
            sql,
            lines,
            unicode,
        }
    }

    fn index_at(&self, line: usize, column: usize) -> Option<usize> {
        let line = line.checked_sub(1)?;
        let (start_byte, start_character, unicode_start) = *self.lines.get(line)?;
        let column = column.checked_sub(1)?;
        let next = self.lines.get(line + 1);
        let end = next.map_or(self.sql.len(), |(next, _, _)| next - 1);
        let unicode_end = next.map_or(self.unicode.len(), |(_, _, count)| *count);
        if unicode_start == unicode_end {
            let byte = start_byte.checked_add(column)?;
            return (byte <= end).then_some(byte);
        }
        let character = start_character.checked_add(column)?;
        let count = self.unicode.partition_point(|(_, end)| *end <= character);
        let extra = count.checked_sub(1).map_or(0, |index| {
            let (byte, character) = self.unicode[index];
            byte - character
        });
        let byte = character.checked_add(extra)?;
        (byte <= end).then_some(byte)
    }

    fn line_col_at(&self, byte: usize) -> Option<(usize, usize)> {
        if !self.sql.is_char_boundary(byte) {
            return None;
        }
        let count = self.unicode.partition_point(|(end, _)| *end <= byte);
        let extra = count.checked_sub(1).map_or(0, |index| {
            let (byte, character) = self.unicode[index];
            byte - character
        });
        let character = byte - extra;
        let line = self.lines.partition_point(|(start, _, _)| *start <= byte) - 1;
        Some((line + 1, character - self.lines[line].1 + 1))
    }
}
fn prev_char(sql: &str, index: usize) -> Option<(usize, char)> {
    let ch = sql.get(..index)?.chars().next_back()?;
    Some((index - ch.len_utf8(), ch))
}

fn is_word_char(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphanumeric()
}

#[cfg(test)]
mod tests;
