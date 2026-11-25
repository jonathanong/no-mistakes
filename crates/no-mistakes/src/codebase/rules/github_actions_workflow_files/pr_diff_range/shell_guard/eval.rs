use super::super::exprs::variable;
use crate::codebase::rules::github_actions_download_integrity::words::Word;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OnMergeGroup {
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

pub(super) fn eval_condition(
    words: &[Word],
    env: &BTreeMap<String, String>,
    mutated_vars: &BTreeSet<String>,
    mutated_all: bool,
) -> OnMergeGroup {
    let [_, open, event, operator, value, close] = words else {
        return OnMergeGroup::Unknown;
    };
    let valid_event_word = match (open.text.as_str(), close.text.as_str()) {
        ("[", "]") => event.quoted_expansion == Some((0, event.text.len())),
        ("[[", "]]") => {
            event.quoted_expansion == Some((0, event.text.len()))
                || event.bare_expansion == Some((0, event.text.len()))
        }
        _ => return OnMergeGroup::Unknown,
    };
    if !valid_event_word {
        return OnMergeGroup::Unknown;
    }
    let Some(name) = variable(&event.text) else {
        return OnMergeGroup::Unknown;
    };
    if mutated_all || mutated_vars.contains(name) || !is_event_name_var(name, env) {
        return OnMergeGroup::Unknown;
    }
    if !is_plain_event_literal(value) {
        return OnMergeGroup::Unknown;
    }
    match (operator.text.as_str(), value.text.as_str()) {
        ("=" | "==", "merge_group") => OnMergeGroup::AlwaysTrue,
        ("=" | "==", _) => OnMergeGroup::AlwaysFalse,
        ("!=", "merge_group") => OnMergeGroup::AlwaysFalse,
        ("!=", _) => OnMergeGroup::AlwaysTrue,
        _ => OnMergeGroup::Unknown,
    }
}

fn is_event_name_var(name: &str, env: &BTreeMap<String, String>) -> bool {
    match env.get(name) {
        Some(value) => value
            .trim()
            .strip_prefix("${{")
            .and_then(|body| body.strip_suffix("}}"))
            .is_some_and(|body| body.trim() == "github.event_name"),
        None => name == "GITHUB_EVENT_NAME",
    }
}

fn is_plain_event_literal(word: &Word) -> bool {
    word.quoted_expansion.is_none()
        && word.bare_expansion.is_none()
        && !word.text.is_empty()
        && word
            .text
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-'))
}

pub(super) fn record_mutations(
    words: &[Word],
    mutated_vars: &mut BTreeSet<String>,
    mutated_all: &mut bool,
) {
    let mut index = 0;
    while let Some(word) = words.get(index) {
        if let Some(name) = assignment_target(&word.text) {
            mutated_vars.insert(name.to_string());
            index += 1;
            continue;
        }
        break;
    }
    while words
        .get(index)
        .is_some_and(|word| matches!(word.text.as_str(), "command" | "builtin"))
    {
        index += 1;
        while let Some(flag) = words.get(index) {
            if !flag.text.starts_with('-') {
                break;
            }
            index += 1;
            if flag.text == "--" {
                break;
            }
        }
    }
    let Some(cmd) = words.get(index) else {
        return;
    };
    if cmd.quoted_expansion.is_some() || cmd.bare_expansion.is_some() {
        *mutated_all = true;
        return;
    }
    let args = &words[index + 1..];
    match cmd.text.as_str() {
        "eval" | "source" | "." => *mutated_all = true,
        "export" | "declare" | "typeset" | "local" | "readonly" | "unset" | "let" => {
            for arg in args {
                if arg.text.starts_with('-') {
                    continue;
                }
                let candidate = assignment_target(&arg.text).unwrap_or(arg.text.as_str());
                if is_ident(candidate) {
                    mutated_vars.insert(candidate.to_string());
                }
            }
        }
        "read" | "mapfile" | "readarray" | "getopts" => {
            for arg in args {
                if !arg.text.starts_with('-') && is_ident(&arg.text) {
                    mutated_vars.insert(arg.text.clone());
                }
            }
        }
        "printf" => {
            for pair in args.windows(2) {
                if pair[0].text == "-v" && is_ident(&pair[1].text) {
                    mutated_vars.insert(pair[1].text.clone());
                }
            }
        }
        _ => {}
    }
}

fn assignment_target(text: &str) -> Option<&str> {
    let (lhs, _) = text.split_once('=')?;
    let name = lhs.strip_suffix('+').unwrap_or(lhs);
    is_ident(name).then_some(name)
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
