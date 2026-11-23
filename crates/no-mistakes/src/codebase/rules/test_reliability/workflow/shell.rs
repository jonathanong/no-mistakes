use crate::codebase::shell_words::{split_quoted_words, Whitespace};

mod lexer;

#[derive(Default)]
struct Loop {
    retries: Option<u64>,
    variable: Option<String>,
    in_body: bool,
    matched_test: bool,
}

pub(super) fn rerun_loops(script: &str, commands: &[Vec<String>]) -> Vec<Option<u64>> {
    let mut loops = Vec::<Loop>::new();
    let mut found = Vec::new();
    for segment in lexer::segments(script) {
        let Some(words) = split_quoted_words(segment, Whitespace::Ascii) else {
            continue;
        };
        let words = words.as_slice();
        match words.first().map(String::as_str) {
            Some("for") => {
                let (variable, retries) = for_limit(words);
                loops.push(Loop {
                    retries,
                    variable,
                    ..Loop::default()
                });
            }
            Some("while" | "until") => {
                loops.push(Loop {
                    matched_test: matching_command(&words[1..], commands, None),
                    ..Loop::default()
                });
            }
            Some("do") => {
                if let Some(frame) = loops.last_mut() {
                    frame.in_body = true;
                    frame.matched_test |=
                        matching_command(&words[1..], commands, frame.variable.as_deref());
                }
            }
            Some("done") => {
                if let Some(frame) = loops.pop() {
                    if frame.matched_test && frame.in_body {
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
                    frame.matched_test |=
                        matching_command(words, commands, frame.variable.as_deref());
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

fn matching_command(words: &[String], prefixes: &[Vec<String>], variable: Option<&str>) -> bool {
    let words = if words.first().is_some_and(|word| word == "!") {
        &words[1..]
    } else {
        words
    };
    let Some(start) = words.iter().position(|word| {
        !word
            .split_once('=')
            .is_some_and(|(name, _)| is_identifier(name))
    }) else {
        return false;
    };
    let words = &words[start..];
    if variable.is_some_and(|var| {
        words.iter().any(|word| {
            word.split('$').skip(1).any(|tail| {
                tail.strip_prefix(var).is_some_and(|rest| {
                    rest.chars()
                        .next()
                        .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
                }) || tail.strip_prefix('{').is_some_and(|rest| {
                    rest.strip_prefix(var)
                        .is_some_and(|rest| rest.starts_with('}'))
                })
            })
        })
    }) {
        return false;
    }
    prefixes.iter().any(|prefix| words.starts_with(prefix))
}

fn is_identifier(word: &str) -> bool {
    let mut chars = word.chars();
    chars
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
