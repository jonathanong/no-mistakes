pub(super) struct InsertSources<'a> {
    sql: &'a str,
    pairs: std::cell::OnceCell<Vec<Word>>,
}

impl<'a> InsertSources<'a> {
    pub(super) fn new(sql: &'a str) -> Self {
        Self {
            sql,
            pairs: std::cell::OnceCell::new(),
        }
    }

    fn pairs(&self) -> &[Word] {
        self.pairs
            .get_or_init(|| keyword_pair_words(self.sql, "insert", "into"))
    }

    pub(super) fn line(&self, n: usize) -> usize {
        self.pairs()
            .get(n.saturating_sub(1))
            .map_or(1, |word| word.line)
    }

    pub(super) fn source(&self, n: usize) -> &str {
        let start = self
            .pairs()
            .get(n.saturating_sub(1))
            .map_or(0, |word| word.start);
        let end = self
            .pairs()
            .get(n)
            .map_or(self.sql.len(), |word| word.start);
        self.sql.get(start..end).unwrap_or_default()
    }
}

pub(super) fn nth_keyword_pair_line(sql: &str, first: &str, second: &str, n: usize) -> usize {
    keyword_pair_lines(sql, first, second)
        .get(n.saturating_sub(1))
        .copied()
        .unwrap_or(1)
}

fn keyword_pair_lines(sql: &str, first: &str, second: &str) -> Vec<usize> {
    keyword_pair_words(sql, first, second)
        .into_iter()
        .map(|word| word.line)
        .collect()
}

fn keyword_pair_words(sql: &str, first: &str, second: &str) -> Vec<Word> {
    let words = words(sql);
    (0..words.len().saturating_sub(1))
        .filter(|&index| eq(&words[index], first) && eq(&words[index + 1], second))
        .map(|index| words[index].clone())
        .collect()
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

/// Line and column (both 1-based) of the last `word` that starts before `end`,
/// ignoring comments and quoted text.
pub(super) fn last_word_position(sql: &str, word: &str, end: usize) -> Option<(usize, usize)> {
    let prefix = sql.get(..end)?;
    let found = words(prefix).into_iter().rfind(|found| eq(found, word))?;
    let line_start = prefix[..found.start]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    Some((
        found.line,
        prefix[line_start..found.start].chars().count() + 1,
    ))
}

#[derive(Clone)]
struct Word {
    start: usize,
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
            // `foo$tag$` is an identifier, not a dollar-quote opener.
            b'$' if index == 0
                || !(bytes[index - 1].is_ascii_alphanumeric()
                    || bytes[index - 1] == b'_'
                    || bytes[index - 1] >= 0x80) =>
            {
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
                    start,
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
