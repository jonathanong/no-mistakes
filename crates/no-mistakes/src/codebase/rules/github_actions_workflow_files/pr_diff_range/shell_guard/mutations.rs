use super::scan;
use crate::codebase::rules::github_actions_download_integrity::words::Word;
use std::collections::BTreeSet;
use std::ops::Range;

pub(super) fn record_mutations(
    script: &str,
    range: Range<usize>,
    heredocs: &[scan::HeredocSpan],
    words: &[Word],
    mutated_vars: &mut BTreeSet<String>,
    mutated_all: &mut bool,
) {
    if scan::has_complex_expansion_in_range(script, range, heredocs) {
        *mutated_all = true;
        return;
    }
    let mut index = 0;
    while words
        .get(index)
        .is_some_and(|word| word.text == "!" && is_literal_word(word))
    {
        index += 1;
    }
    while let Some(word) = words.get(index) {
        if let Some((lhs, rhs)) = word.text.split_once('=') {
            let name = lhs.strip_suffix('+').unwrap_or(lhs);
            if is_plain_target(word, name) && !rhs.starts_with('(') {
                mutated_vars.insert(name.to_string());
                index += 1;
                continue;
            }
            *mutated_all = true;
            return;
        }
        break;
    }
    while words.get(index).is_some_and(|word| {
        is_literal_word(word) && matches!(word.text.as_str(), "command" | "builtin")
    }) {
        index += 1;
        while let Some(flag) = words.get(index) {
            if !flag.text.starts_with('-') {
                break;
            }
            if !is_literal_word(flag) {
                *mutated_all = true;
                return;
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
    if !is_literal_word(cmd) {
        *mutated_all = true;
        return;
    }
    let args = &words[index + 1..];
    match cmd.text.as_str() {
        ":" | "true" | "false" | "echo" | "set" | "shopt" | "exit" | "return" | "cat" | "git" => {}
        "[" | "test" => record_test_builtin(args, mutated_all),
        "[[" => record_double_bracket(args, mutated_vars, mutated_all),
        "printf" => record_printf(args, mutated_vars, mutated_all),
        "export" | "readonly" | "local" | "declare" | "typeset" | "unset" => {
            record_target_builtin(args, mutated_vars, mutated_all);
        }
        _ => *mutated_all = true,
    }
}

fn record_test_builtin(args: &[Word], mutated_all: &mut bool) {
    for pair in args.windows(2) {
        if pair[0].text == "-v" && (!is_literal_word(&pair[1]) || pair[1].text.contains('[')) {
            *mutated_all = true;
            return;
        }
        if !is_literal_word(&pair[0]) && pair[1].text.contains('[') {
            *mutated_all = true;
            return;
        }
    }
}

fn record_double_bracket(
    args: &[Word],
    mutated_vars: &mut BTreeSet<String>,
    mutated_all: &mut bool,
) {
    for arg in args {
        match arg.text.as_str() {
            "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge" | "-v" | "-R" => {
                *mutated_all = true;
                return;
            }
            "=~" => {
                mutated_vars.insert("BASH_REMATCH".to_string());
            }
            _ => {}
        }
    }
}

fn record_printf(args: &[Word], mutated_vars: &mut BTreeSet<String>, mutated_all: &mut bool) {
    let Some(first) = args.first() else {
        return;
    };
    if !is_literal_word(first) {
        *mutated_all = true;
    } else if first.text == "-v" {
        match args.get(1) {
            Some(target) if is_literal_word(target) && is_ident(&target.text) => {
                mutated_vars.insert(target.text.clone());
            }
            _ => *mutated_all = true,
        }
    } else if first.text.starts_with('-') && first.text != "--" {
        *mutated_all = true;
    }
}

fn record_target_builtin(
    args: &[Word],
    mutated_vars: &mut BTreeSet<String>,
    mutated_all: &mut bool,
) {
    let mut past_flags = false;
    for arg in args {
        if !past_flags && arg.text == "--" && is_literal_word(arg) {
            past_flags = true;
            continue;
        }
        if !past_flags && arg.text.starts_with('-') {
            if !is_literal_word(arg) || arg.text.contains(['n', 'f', 'i']) {
                *mutated_all = true;
                return;
            }
            continue;
        }
        past_flags = true;
        if arg.expansion_may_split {
            *mutated_all = true;
            return;
        }
        let (target, is_compound) = match arg.text.split_once('=') {
            Some((lhs, rhs)) => (lhs.strip_suffix('+').unwrap_or(lhs), rhs.starts_with('(')),
            None => (arg.text.as_str(), false),
        };
        if is_plain_target(arg, target) && !is_compound {
            mutated_vars.insert(target.to_string());
        } else {
            *mutated_all = true;
            return;
        }
    }
}

fn is_literal_word(word: &Word) -> bool {
    word.quoted_expansion.is_none()
        && word.bare_expansion.is_none()
        && !word.expansion_may_split
        && !word.text.contains(['$', '`'])
}

fn is_plain_target(word: &Word, name: &str) -> bool {
    is_ident(name)
        && word
            .quoted_expansion
            .is_none_or(|(pos, _)| pos >= name.len())
        && word.bare_expansion.is_none_or(|(pos, _)| pos >= name.len())
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
