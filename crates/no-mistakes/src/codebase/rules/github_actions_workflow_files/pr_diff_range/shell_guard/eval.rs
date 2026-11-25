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
