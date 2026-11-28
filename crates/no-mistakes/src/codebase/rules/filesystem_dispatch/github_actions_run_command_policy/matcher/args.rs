use super::dynamic_match::dynamic_word_could_match_literal;
use super::flag_match::{match_flag_pair, match_single_flag};
use super::ParsedInvocation;
use crate::codebase::rules::github_actions_download_integrity::words::Word;
use crate::codebase::rules::github_actions_run_command_policy::options::ArgMatcher;

pub(super) enum ArgMatch {
    Matched,
    NotMatched,
    DynamicValue { token: String, line: usize },
}

pub(super) fn match_invocation(
    inv: &ParsedInvocation<'_>,
    matcher: &ArgMatcher,
    value_flags: &[String],
    require_value: bool,
) -> ArgMatch {
    let is_flag = matcher
        .tokens
        .first()
        .is_some_and(|token| is_option_token(token));
    if !is_flag {
        return super::positional::match_positionals(
            &inv.subcommand_args,
            &matcher.tokens,
            inv.command_dynamic,
            value_flags,
        );
    }
    let global = match_flag_slice(
        &inv.global_args,
        matcher,
        value_flags,
        require_value,
        inv.command_dynamic,
    );
    if matches!(global, ArgMatch::Matched) {
        return ArgMatch::Matched;
    }
    let sub = match_flag_slice(
        &inv.subcommand_args,
        matcher,
        value_flags,
        require_value,
        inv.command_dynamic,
    );
    match (global, sub) {
        (_, ArgMatch::Matched) => ArgMatch::Matched,
        (ArgMatch::DynamicValue { token, line }, _)
        | (_, ArgMatch::DynamicValue { token, line }) => ArgMatch::DynamicValue { token, line },
        _ => ArgMatch::NotMatched,
    }
}

fn match_flag_slice(
    args: &[&Word],
    matcher: &ArgMatcher,
    value_flags: &[String],
    require_value: bool,
    command_dynamic: bool,
) -> ArgMatch {
    let (stop, consumed) = option_terminator(args, value_flags);
    let region = &args[..stop];
    if matcher.tokens.len() > 2 {
        return super::sequence::match_token_sequence_skipping(
            region,
            &matcher.tokens,
            command_dynamic,
            &consumed[..stop],
        );
    }
    let mut index = 0;
    let mut dynamic_hit = None;
    while index < region.len() {
        let word = region[index];
        if is_option_token(&word.text) {
            let consumes_next = !word.text.contains('=') && takes_value(&word.text, value_flags);
            let next = consumes_next
                .then(|| region.get(index + 1).copied())
                .flatten();
            let outcome = if matcher.tokens.len() == 1 {
                match_single_flag(
                    word,
                    next,
                    &matcher.tokens[0],
                    value_flags,
                    require_value,
                    command_dynamic,
                )
            } else {
                match_flag_pair(
                    word,
                    next,
                    &matcher.tokens[0],
                    &matcher.tokens[1],
                    value_flags,
                    command_dynamic,
                )
            };
            match outcome {
                ArgMatch::Matched => return ArgMatch::Matched,
                ArgMatch::DynamicValue { token, line } => {
                    dynamic_hit.get_or_insert((token, line));
                }
                ArgMatch::NotMatched => {}
            }
            index += 1 + usize::from(consumes_next);
        } else {
            let could_be_flag = matcher.tokens.first().is_some_and(|token| {
                dynamic_word_could_match_literal(word, command_dynamic, token)
            });
            let pair_value_ok = matcher.tokens.len() <= 1
                || matcher.tokens.get(1).is_some_and(|expected| {
                    region.get(index + 1).is_some_and(|next| {
                        next.text == *expected
                            || dynamic_word_could_match_literal(next, command_dynamic, expected)
                    })
                });
            if could_be_flag && pair_value_ok {
                dynamic_hit.get_or_insert_with(|| (word.text.clone(), word.line));
            }
            index += 1;
        }
    }
    match dynamic_hit {
        Some((token, line)) => ArgMatch::DynamicValue { token, line },
        None => ArgMatch::NotMatched,
    }
}

fn option_terminator(args: &[&Word], value_flags: &[String]) -> (usize, Vec<bool>) {
    let mut consumed = vec![false; args.len()];
    let mut index = 0;
    while index < args.len() {
        let word = args[index];
        if word.text == "--" {
            return (index, consumed);
        }
        if is_option_token(&word.text)
            && !word.text.contains('=')
            && takes_value(&word.text, value_flags)
        {
            if index + 1 < args.len() {
                consumed[index + 1] = true;
            }
            index += 2;
            continue;
        }
        index += 1;
    }
    (args.len(), consumed)
}

pub(super) fn extract_positionals<'a>(args: &[&'a Word], value_flags: &[String]) -> Vec<&'a Word> {
    let mut out = Vec::new();
    let mut index = 0;
    let mut after_dd = false;
    while index < args.len() {
        let word = args[index];
        if after_dd {
            out.push(word);
            index += 1;
            continue;
        }
        if word.text == "--" {
            after_dd = true;
            index += 1;
            continue;
        }
        if is_option_token(&word.text) {
            let skip_value = !word.text.contains('=') && takes_value(&word.text, value_flags);
            index += 1 + usize::from(skip_value);
            continue;
        }
        out.push(word);
        index += 1;
    }
    out
}

pub(super) fn is_option_token(text: &str) -> bool {
    text.starts_with('-') && text.len() > 1 && text != "--"
}

pub(super) fn takes_value(flag: &str, value_flags: &[String]) -> bool {
    value_flags.iter().any(|entry| entry == flag)
}

pub(super) fn unwrap_group(text: &str) -> &str {
    text.trim_start_matches(['(', '{'])
        .trim_end_matches([')', '}'])
}
