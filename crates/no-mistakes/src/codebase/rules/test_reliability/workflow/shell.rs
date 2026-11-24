use crate::codebase::shell_words::{split_quoted_words, Whitespace};

mod lexer;

#[derive(Default)]
struct Loop {
    retries: Option<u64>,
    variables: Vec<String>,
    in_body: bool,
    matched_test: bool,
    uses_input: bool,
}

pub(super) fn rerun_loops(script: &str, commands: &[Vec<String>]) -> Vec<Option<u64>> {
    let mut loops = Vec::<Loop>::new();
    let mut found = Vec::new();
    for segment in lexer::segments(script) {
        let Some(words) = split_quoted_words(segment, Whitespace::Ascii) else {
            continue;
        };
        let words = words.as_slice();
        let first = words.first().map(String::as_str);
        for frame in loops.iter_mut().filter(|frame| frame.in_body || first == Some("do")) {
            frame.uses_input |= lexer::expands_any(segment, &frame.variables);
        }
        match first {
            Some("for") => {
                let (variable, retries) = for_limit(words);
                loops.push(Loop {
                    retries,
                    variables: variable.into_iter().collect(),
                    ..Loop::default()
                });
            }
            Some("while" | "until") => {
                loops.push(Loop {
                    variables: read_variables(&words[1..]),
                    matched_test: matching_command(&words[1..], commands),
                    ..Loop::default()
                });
            }
            Some("do") => {
                if let Some(frame) = loops.last_mut() {
                    frame.in_body = true;
                    frame.matched_test |= matching_command(&words[1..], commands);
                }
            }
            Some("done") => {
                if let Some(frame) = loops.pop() {
                    if frame.matched_test && frame.in_body && !frame.uses_input {
                        found.push(frame.retries);
                    }
                }
            }
            _ => {
                if let Some(frame) = loops.last_mut().filter(|frame| frame.in_body) {
                    let words = match words.first().map(String::as_str) {
                        Some("if" | "elif" | "then" | "else") => &words[1..],
                        _ => words,
                    };
                    frame.matched_test |= matching_command(words, commands);
                }
            }
        }
    }
    found
}

fn for_limit(words: &[String]) -> (Option<String>, Option<u64>) {
    let variable = words.get(1).filter(|word| is_identifier(word)).cloned();
    let Some(items) = words
        .get(3..)
        .filter(|_| variable.is_some() && words[2] == "in")
    else {
        return (variable, None);
    };
    let count = match items {
        [range] if range.starts_with('{') && range.ends_with('}') => range[1..range.len() - 1]
            .split_once("..")
            .and_then(|(start, end)| {
                let start = start.parse::<i64>().ok()?;
                let end = end.parse::<i64>().ok()?;
                start.abs_diff(end).checked_add(1)
            }),
        items
            if items
                .iter()
                .all(|item| !item.contains(['$', '`', '*', '?', '[', '{'])) =>
        {
            Some(items.len() as u64)
        }
        _ => None,
    };
    (variable, count.map(|count| count.saturating_sub(1)))
}

fn read_variables(words: &[String]) -> Vec<String> {
    let Some(start) = words.iter().position(|word| !is_assignment(word)) else {
        return Vec::new();
    };
    let words = &words[start..];
    if words.first().map(String::as_str) != Some("read") {
        return Vec::new();
    }
    let mut vars = Vec::new();
    let mut end_of_options = false;
    let mut iter = words[1..].iter();
    while let Some(word) = iter.next() {
        if !end_of_options && word == "--" {
            end_of_options = true;
            continue;
        }
        if !end_of_options && word.starts_with('-') && word.len() > 1 {
            if word == "-a" {
                if let Some(name) = iter.next().filter(|name| is_identifier(name)) {
                    vars.push(name.clone());
                }
            } else if matches!(
                word.as_bytes().last(),
                Some(b'd' | b'i' | b'n' | b'N' | b'p' | b't' | b'u')
            ) {
                iter.next();
            }
            continue;
        }
        if is_redirection_operator(word) {
            iter.next();
            continue;
        }
        if word.starts_with(['<', '>']) {
            continue;
        }
        if is_identifier(word) {
            vars.push(word.clone());
        }
    }
    if vars.is_empty() {
        vars.push("REPLY".into());
    }
    vars
}

fn matching_command(words: &[String], prefixes: &[Vec<String>]) -> bool {
    let words = if words.first().is_some_and(|word| word == "!") {
        &words[1..]
    } else {
        words
    };
    let Some(start) = words.iter().position(|word| !is_assignment(word)) else {
        return false;
    };
    let command = &words[start..];
    prefixes.iter().any(|prefix| command.starts_with(prefix))
}

fn is_assignment(word: &str) -> bool {
    word.split_once('=')
        .is_some_and(|(name, _)| is_identifier(name))
}

fn is_redirection_operator(word: &str) -> bool {
    matches!(word, "<" | ">" | ">>" | "<<" | "<<<" | "<&" | ">&" | "<>")
}

fn is_identifier(word: &str) -> bool {
    let mut chars = word.chars();
    chars
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
