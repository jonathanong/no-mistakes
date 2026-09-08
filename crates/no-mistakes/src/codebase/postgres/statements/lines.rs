pub(super) fn nth_insert_line(sql: &str, n: usize) -> usize {
    nth_keyword_pair_line(sql, "insert", "into", n)
}

pub(super) fn nth_insert_source(sql: &str, n: usize) -> String {
    let lines = keyword_pair_lines(sql, "insert", "into");
    let start = lines.get(n.saturating_sub(1)).copied().unwrap_or(1);
    slice_lines(sql, start, lines.get(n).copied())
}

pub(super) fn nth_keyword_pair_line(sql: &str, first: &str, second: &str, n: usize) -> usize {
    keyword_pair_lines(sql, first, second)
        .get(n.saturating_sub(1))
        .copied()
        .unwrap_or(1)
}

fn keyword_pair_lines(sql: &str, first: &str, second: &str) -> Vec<usize> {
    let words = words(sql);
    (0..words.len().saturating_sub(1))
        .filter(|&index| eq(&words[index], first) && eq(&words[index + 1], second))
        .map(|index| words[index].line)
        .collect()
}

fn slice_lines(sql: &str, start: usize, end: Option<usize>) -> String {
    let end = end.filter(|end| *end > start).unwrap_or(usize::MAX);
    sql.lines()
        .enumerate()
        .filter(|(index, _)| {
            let line = index + 1;
            line >= start && line < end
        })
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn line_containing(source: &str, parts: &[&str]) -> usize {
    source
        .lines()
        .enumerate()
        .find(|(_, line)| {
            let lower = line.to_ascii_lowercase();
            parts
                .iter()
                .all(|part| lower.contains(&part.to_ascii_lowercase()))
        })
        .map(|(index, _)| index + 1)
        .unwrap_or(1)
}

struct Word {
    line: usize,
    text: String,
}

fn words(sql: &str) -> Vec<Word> {
    let bytes = sql.as_bytes();
    let mut index = 0usize;
    let mut line = 1usize;
    let mut out = Vec::new();
    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                line += 1;
                index += 1;
            }
            b'-' if bytes.get(index + 1) == Some(&b'-') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index, &mut line);
            }
            quote @ (b'\'' | b'"') => index = skip_quoted(bytes, index, quote, &mut line),
            b'$' => {
                if let Some(end) = skip_dollar(bytes, index, &mut line) {
                    index = end;
                } else {
                    index += 1;
                }
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = index;
                let start_line = line;
                index += 1;
                while index < bytes.len()
                    && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
                {
                    index += 1;
                }
                out.push(Word {
                    line: start_line,
                    text: sql[start..index].to_string(),
                });
            }
            _ => index += 1,
        }
    }
    out
}

fn skip_block_comment(bytes: &[u8], mut index: usize, line: &mut usize) -> usize {
    index += 2;
    let mut depth = 1i32;
    while index < bytes.len() && depth > 0 {
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            depth += 1;
            continue;
        }
        if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
            index += 2;
            depth -= 1;
            continue;
        }
        if bytes[index] == b'\n' {
            *line += 1;
        }
        index += 1;
    }
    index.min(bytes.len())
}

fn skip_quoted(bytes: &[u8], mut index: usize, quote: u8, line: &mut usize) -> usize {
    index += 1;
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            *line += 1;
        }
        if bytes[index] == quote {
            if quote == b'\'' && bytes.get(index + 1) == Some(&b'\'') {
                index += 2;
                continue;
            }
            return index + 1;
        }
        index += 1;
    }
    index
}

fn skip_dollar(bytes: &[u8], start: usize, line: &mut usize) -> Option<usize> {
    let mut index = start + 1;
    while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_') {
        index += 1;
    }
    if index >= bytes.len() || bytes[index] != b'$' {
        return None;
    }
    let tag_len = index + 1 - start;
    index += 1;
    while index + tag_len <= bytes.len() {
        if bytes[index] == b'\n' {
            *line += 1;
        }
        if bytes[index..index + tag_len] == bytes[start..start + tag_len] {
            return Some(index + tag_len);
        }
        index += 1;
    }
    Some(bytes.len())
}

fn eq(word: &Word, expected: &str) -> bool {
    word.text.eq_ignore_ascii_case(expected)
}
