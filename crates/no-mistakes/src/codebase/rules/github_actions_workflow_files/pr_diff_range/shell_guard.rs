use super::exprs::variable;
use crate::codebase::rules::github_actions_download_integrity::words::{parse, Command, Word};
use std::{collections::BTreeMap, ops::Range};

pub(super) fn excluded_on_merge_group(
    script: &str,
    env: &BTreeMap<String, String>,
) -> Option<Range<usize>> {
    if !script.split_ascii_whitespace().any(|word| word == "if") {
        return None;
    }
    let parsed = parse(script);
    let mut commands = parsed.commands.iter();
    let condition = commands.next()?;
    let take_then = merge_group_takes_then(&condition.words, env)?;
    let positions: Vec<_> = script.char_indices().map(|(index, _)| index).collect();
    let start = |command: &Command| positions[command.anchor];
    let source = |command: &Command| script[start(command)..].trim_start();
    let then = commands.next()?;
    let test = &script[start(condition)..start(then)];
    if keyword(source(condition)) != "if"
        || keyword(source(then)) != "then"
        || test.contains(['<', '>', '|', '&'])
        || nested_control(source(then)[4..].trim_start())
    {
        return None;
    }
    let mut otherwise = None;
    for command in commands {
        let body = source(command);
        let body = match keyword(body) {
            "else" if otherwise.is_none() => {
                otherwise = Some(start(command));
                body[4..].trim_start()
            }
            "fi" => {
                return if take_then {
                    otherwise.map(|else_start| else_start..start(command))
                } else {
                    Some(start(then)..otherwise.unwrap_or_else(|| start(command)))
                };
            }
            _ => body,
        };
        if nested_control(body) {
            return None;
        }
    }
    None
}

fn keyword(source: &str) -> &str {
    source
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ';' | '|' | '&'))
        .next()
        .unwrap_or_default()
}

fn nested_control(source: &str) -> bool {
    let source = source.trim_start_matches(|ch: char| {
        ch.is_whitespace() || matches!(ch, '|' | '&' | '!')
    });
    let word = keyword(source);
    source.starts_with(['(', '{'])
        || matches!(word, "if" | "elif" | "else" | "case" | "while" | "until" | "for" | "select" | "function")
        || word.ends_with("()")
}

fn merge_group_takes_then(words: &[Word], env: &BTreeMap<String, String>) -> Option<bool> {
    let [if_word, open, event, operator, value, close] = words else {
        return None;
    };
    if if_word.text != "if"
        || !matches!((open.text.as_str(), close.text.as_str()), ("[", "]") | ("[[", "]]"))
        || event.quoted_expansion != Some((0, event.text.len()))
    {
        return None;
    }
    let name = variable(&event.text)?;
    if !env.get(name).is_some_and(|value| {
        value.trim().strip_prefix("${{").and_then(|body| body.strip_suffix("}}"))
            .is_some_and(|body| body.trim() == "github.event_name")
    }) {
        return None;
    }
    match (operator.text.as_str(), value.text.as_str()) {
        ("=" | "==", "merge_group") | ("!=", "pull_request" | "pull_request_target") => Some(true),
        ("=" | "==", "pull_request" | "pull_request_target") | ("!=", "merge_group") => Some(false),
        _ => None,
    }
}
