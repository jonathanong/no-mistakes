pub(super) fn nth_insert_line(sql: &str, n: usize) -> usize {
    nth_keyword_pair_line(sql, "insert", "into", n)
}

pub(super) fn nth_keyword_pair_line(sql: &str, first: &str, second: &str, n: usize) -> usize {
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
    let mut words = Vec::new();
    for (index, line) in sql.lines().enumerate() {
        for token in line.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
            if !token.is_empty() {
                words.push(Word {
                    line: index + 1,
                    text: token.to_string(),
                });
            }
        }
    }
    words
}

fn eq(word: &Word, expected: &str) -> bool {
    word.text.eq_ignore_ascii_case(expected)
}
