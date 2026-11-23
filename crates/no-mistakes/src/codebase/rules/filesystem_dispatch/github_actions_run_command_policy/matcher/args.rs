use super::ParsedInvocation;
use crate::codebase::rules::github_actions_download_integrity::words::Word;
use crate::codebase::rules::github_actions_run_command_policy::options::ArgMatcher;

pub(super) fn matches_invocation(
    inv: &ParsedInvocation<'_>,
    matcher: &ArgMatcher,
    value_flags: &[String],
) -> bool {
    matches_slice(&inv.global_args, matcher, value_flags)
        || matches_slice(&inv.subcommand_args, matcher, value_flags)
}

fn matches_slice(args: &[&Word], matcher: &ArgMatcher, value_flags: &[String]) -> bool {
    let is_flag = matcher
        .tokens
        .first()
        .is_some_and(|token| is_option_token(token));
    let region = if is_flag {
        let stop = args
            .iter()
            .position(|word| word.text == "--")
            .unwrap_or(args.len());
        &args[..stop]
    } else {
        args
    };
    if matcher.tokens.len() == 1 {
        let target = &matcher.tokens[0];
        return region
            .iter()
            .any(|word| single_matches(&word.text, target, value_flags));
    }
    if region.windows(matcher.tokens.len()).any(|window| {
        window
            .iter()
            .zip(&matcher.tokens)
            .all(|(w, s)| w.text == *s)
    }) {
        return true;
    }
    if matcher.tokens.len() == 2 && is_flag {
        let flag = &matcher.tokens[0];
        let value = &matcher.tokens[1];
        let eq = format!("{flag}={value}");
        if region.iter().any(|word| word.text == eq) {
            return true;
        }
        if is_short_flag(flag) && takes_value(flag, value_flags) {
            let attached = format!("{flag}{value}");
            if region.iter().any(|word| word.text == attached) {
                return true;
            }
        }
    }
    false
}

fn single_matches(arg: &str, target: &str, value_flags: &[String]) -> bool {
    if arg == target {
        return true;
    }
    if target.starts_with("--") && target.len() > 2 {
        return arg
            .strip_prefix(target)
            .is_some_and(|rest| rest.starts_with('='));
    }
    if is_short_flag(target) {
        if let Some(rest) = arg.strip_prefix(target) {
            return rest.starts_with('=') || (!rest.is_empty() && takes_value(target, value_flags));
        }
    }
    false
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

fn is_short_flag(text: &str) -> bool {
    text.len() == 2 && text.starts_with('-') && !text.starts_with("--")
}

pub(super) fn takes_value(flag: &str, value_flags: &[String]) -> bool {
    value_flags.iter().any(|entry| entry == flag)
}

pub(super) fn unwrap_group(text: &str) -> &str {
    text.trim_start_matches(['(', '{'])
        .trim_end_matches([')', '}'])
}
