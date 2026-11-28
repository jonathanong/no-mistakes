use super::options::{ArgMatcher, CompiledPolicy};
use crate::codebase::rules::github_actions_download_integrity::words::{self, Word};

mod args;
mod dynamic_match;
mod evaluate;
mod flag_match;
mod positional;
mod sequence;

use args::{is_option_token, takes_value, unwrap_group};
pub(super) use evaluate::evaluate_policy;
use positional::extract_positionals;

pub(super) enum MatchOutcome<'a> {
    NoMatch,
    DynamicSubcommand { token: String, line: usize },
    Matched(ParsedInvocation<'a>),
}

pub(super) struct ParsedInvocation<'a> {
    pub(super) cmd_line: usize,
    pub(super) command_dynamic: bool,
    pub(super) global_args: Vec<&'a Word>,
    pub(super) subcommand_args: Vec<&'a Word>,
    pub(super) positionals: Vec<&'a Word>,
    pub(super) splitting_word: Option<(String, usize)>,
    pub(super) flag_splitting_word: Option<(String, usize)>,
}

pub(super) fn match_command<'a>(
    words: &'a [Word],
    cmd_at: usize,
    command_dynamic: bool,
    policy: &CompiledPolicy,
) -> MatchOutcome<'a> {
    let base = words::basename(unwrap_group(&words[cmd_at].text));
    if base != policy.command[0] {
        return MatchOutcome::NoMatch;
    }
    let mut global_args = Vec::new();
    let mut idx = cmd_at + 1;
    let mut sub_idx = 1;
    while idx < words.len() && sub_idx < policy.command.len() {
        let word = &words[idx];
        if word.text == "--" {
            return MatchOutcome::NoMatch;
        }
        if is_option_token(&word.text) {
            global_args.push(word);
            if !word.text.contains('=') && takes_value(&word.text, &policy.value_flags) {
                idx += 1;
                if let Some(value) = words.get(idx) {
                    global_args.push(value);
                }
            }
            idx += 1;
            continue;
        }
        if is_dynamic_word(word, command_dynamic) {
            return MatchOutcome::DynamicSubcommand {
                token: word.text.clone(),
                line: word.line,
            };
        }
        if word.text == policy.command[sub_idx] {
            sub_idx += 1;
            idx += 1;
        } else {
            return MatchOutcome::NoMatch;
        }
    }
    if sub_idx < policy.command.len() {
        return MatchOutcome::NoMatch;
    }
    let subcommand_args: Vec<&Word> = words[idx..].iter().collect();
    let positionals = extract_positionals(&subcommand_args, &policy.value_flags);
    let splitting_word = splitting_at(global_args.iter().chain(subcommand_args.iter()).copied());
    let flag_stop = option_end(&subcommand_args, &policy.value_flags);
    let flag_splitting_word = splitting_at(
        global_args
            .iter()
            .copied()
            .chain(subcommand_args[..flag_stop].iter().copied()),
    );
    MatchOutcome::Matched(ParsedInvocation {
        cmd_line: words[cmd_at].line,
        command_dynamic,
        global_args,
        subcommand_args,
        positionals,
        splitting_word,
        flag_splitting_word,
    })
}

fn splitting_at<'a>(mut words: impl Iterator<Item = &'a Word>) -> Option<(String, usize)> {
    words
        .find(|word| is_word_splitting_expansion(word))
        .map(|word| (word.text.clone(), word.line))
}

fn option_end(args: &[&Word], value_flags: &[String]) -> usize {
    let mut index = 0;
    while index < args.len() {
        let word = args[index];
        if word.text == "--" {
            return index;
        }
        let takes = is_option_token(&word.text)
            && !word.text.contains('=')
            && takes_value(&word.text, value_flags);
        index += 1 + usize::from(takes);
    }
    args.len()
}

pub(super) fn is_dynamic_word(word: &Word, command_dynamic: bool) -> bool {
    word.bare_expansion.is_some()
        || word.quoted_expansion.is_some()
        || word.expansion_may_split
        || word.text.contains("${{")
        || (command_dynamic && (word.text.contains('$') || word.text.contains('`')))
}

pub(super) fn flags_only(matchers: &[ArgMatcher]) -> bool {
    !matchers.is_empty()
        && matchers.iter().all(|matcher| {
            matcher
                .tokens
                .first()
                .is_some_and(|token| is_option_token(token))
        })
}

pub(super) fn split_hit(inv: &ParsedInvocation<'_>, flags: bool) -> Option<(String, usize)> {
    if flags {
        inv.flag_splitting_word.clone()
    } else {
        inv.splitting_word.clone()
    }
}

fn is_word_splitting_expansion(word: &Word) -> bool {
    word.expansion_may_split
        || word.text.contains("$@")
        || word.text.contains("$*")
        || word.text.contains("[@]}")
        || word.text.contains("[*]}")
}
