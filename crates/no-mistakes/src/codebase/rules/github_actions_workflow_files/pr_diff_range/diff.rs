use super::exprs::{classify, End};
use crate::codebase::rules::github_actions_workflow_files::shell_words::{commands, Word};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Form {
    TwoDot,
    ThreeDot,
    Args,
}

pub(super) struct Range {
    pub(super) line: usize,
    pub(super) left: End,
    pub(super) right: End,
    pub(super) form: Form,
}

const LEADING: &[&str] = &[
    "do", "then", "else", "elif", "if", "!", "{", "(", "time", "sudo", "command", "env", "nohup",
    "exec",
];

/// Flags of `git diff` that take the next word as a value. A command using
/// one is not judged because its positional words are ambiguous.
const VALUE_FLAGS: &[&str] = &[
    "-U",
    "-O",
    "-S",
    "-G",
    "-l",
    "--output",
    "--unified",
    "--diff-algorithm",
    "--line-prefix",
    "--src-prefix",
    "--dst-prefix",
    "--stat-width",
    "--inter-hunk-context",
];

pub(super) fn ranges(text: &str, exprs: &[String], env: &BTreeMap<String, String>) -> Vec<Range> {
    commands(text)
        .iter()
        .filter_map(|command| range(command, exprs, env))
        .collect()
}

fn range(command: &[Word], exprs: &[String], env: &BTreeMap<String, String>) -> Option<Range> {
    let mut words: Vec<&str> = command
        .iter()
        .flat_map(|word| word.text.split_whitespace())
        .collect();
    let first = words.first_mut()?;
    *first = first.rsplit("$(").next().unwrap_or(first);
    *first = first.trim_start_matches(['`', '(']);
    let mut index = 0;
    while index < words.len() && is_leading(words[index]) {
        index += 1;
    }
    if words.get(index)?.rsplit('/').next() != Some("git") {
        return None;
    }
    index += 1;
    while index < words.len() && words[index].starts_with('-') {
        let skip = matches!(
            words[index],
            "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace"
        );
        index += 1 + usize::from(skip);
    }
    if *words.get(index)? != "diff" {
        return None;
    }
    let mut positional = Vec::new();
    for word in &words[index + 1..] {
        let word = word.trim_end_matches([')', '`']);
        if word == "--" {
            break;
        }
        if VALUE_FLAGS.contains(&word) {
            return None;
        }
        if !word.starts_with('-') {
            positional.push(word);
        }
    }
    let (left, right, form) = match positional.as_slice() {
        [one] => one
            .split_once("...")
            .map(|(a, b)| (a, b, Form::ThreeDot))
            .or_else(|| one.split_once("..").map(|(a, b)| (a, b, Form::TwoDot)))?,
        [a, b] if !a.contains("..") && !b.contains("..") => (*a, *b, Form::Args),
        _ => return None,
    };
    Some(Range {
        line: command[0].line,
        left: classify(left, exprs, env),
        right: classify(right, exprs, env),
        form,
    })
}

fn is_leading(text: &str) -> bool {
    LEADING.contains(&text)
        || text.split_once('=').is_some_and(|(name, _)| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        })
}
