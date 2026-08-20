struct Word {
    line: usize,
    text: String,
}

pub(super) fn nth_create_index_line(sql: &str, n: usize) -> usize {
    let words = words(sql);
    let mut found = 0usize;
    for index in 0..words.len() {
        if create_index_at(&words, index).is_some() {
            found += 1;
            if found == n {
                return words[index].line;
            }
        }
    }
    1
}

pub(super) fn nth_drop_index_line(sql: &str, n: usize) -> usize {
    nth_keyword_pair_line(sql, "drop", "index", n)
}

pub(super) fn nth_drop_table_line(sql: &str, n: usize) -> usize {
    nth_keyword_pair_line(sql, "drop", "table", n)
}

fn nth_keyword_pair_line(sql: &str, first: &str, second: &str, n: usize) -> usize {
    let words = words(sql);
    let mut found = 0usize;
    for index in 0..words.len().saturating_sub(1) {
        if eq(&words[index], first) && eq(&words[index + 1], second) {
            found += 1;
            if found == n {
                return words[index].line;
            }
        }
    }
    1
}

fn create_index_at(words: &[Word], index: usize) -> Option<usize> {
    if !eq(&words[index], "create") {
        return None;
    }
    if words.get(index + 1).is_some_and(|word| eq(word, "index")) {
        return Some(index);
    }
    if words.get(index + 1).is_some_and(|word| eq(word, "unique"))
        && words.get(index + 2).is_some_and(|word| eq(word, "index"))
    {
        return Some(index);
    }
    None
}

fn eq(word: &Word, expected: &str) -> bool {
    word.text.eq_ignore_ascii_case(expected)
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
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/')
                {
                    if bytes[index] == b'\n' {
                        line += 1;
                    }
                    index += 1;
                }
                index = index.saturating_add(2).min(bytes.len());
            }
            quote @ (b'\'' | b'"') => index = skip_quoted(bytes, index, quote, &mut line),
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

#[cfg(test)]
mod tests;
