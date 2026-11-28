use super::args::{is_option_token, takes_value, ArgMatch};
use super::dynamic_match::{
    could_be_value_flag, dynamic_word_could_match_literal, is_expanded_word,
};
use super::sequence::match_token_sequence_masked;
use crate::codebase::rules::github_actions_download_integrity::words::Word;

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

pub(super) fn match_positionals(
    args: &[&Word],
    tokens: &[String],
    command_dynamic: bool,
    value_flags: &[String],
) -> ArgMatch {
    let (words, indefinite) = positional_words(args, command_dynamic, value_flags);
    if tokens.len() == 1 {
        return match_one(&words, &indefinite, &tokens[0], command_dynamic);
    }
    match_token_sequence_masked(&words, tokens, command_dynamic, &[], &indefinite)
}

fn match_one(
    words: &[&Word],
    indefinite: &[bool],
    target: &str,
    command_dynamic: bool,
) -> ArgMatch {
    let mut dynamic_hit = None;
    for (index, word) in words.iter().enumerate() {
        let uncertain = indefinite.get(index).copied().unwrap_or(false);
        if word.text == target && !is_expanded_word(word) {
            if !uncertain {
                return ArgMatch::Matched;
            }
            dynamic_hit.get_or_insert_with(|| (word.text.clone(), word.line));
            continue;
        }
        if dynamic_word_could_match_literal(word, command_dynamic, target) {
            dynamic_hit.get_or_insert_with(|| (word.text.clone(), word.line));
        }
    }
    match dynamic_hit {
        Some((token, line)) => ArgMatch::DynamicValue { token, line },
        None => ArgMatch::NotMatched,
    }
}

fn positional_words<'a>(
    args: &[&'a Word],
    command_dynamic: bool,
    value_flags: &[String],
) -> (Vec<&'a Word>, Vec<bool>) {
    let mut words = Vec::new();
    let mut indefinite = Vec::new();
    let mut index = 0;
    let mut after_terminator = false;
    let mut opaque = false;
    while index < args.len() {
        let word = args[index];
        if !opaque && !after_terminator && word.text == "--" {
            after_terminator = true;
            index += 1;
            continue;
        }
        if !opaque && !after_terminator && is_option_token(&word.text) {
            let skip_value = !word.text.contains('=') && takes_value(&word.text, value_flags);
            if !skip_value && could_be_value_flag(word, command_dynamic, value_flags) {
                opaque = true;
            }
            index += 1 + usize::from(skip_value);
            continue;
        }
        words.push(word);
        indefinite.push(opaque);
        if !after_terminator && could_be_value_flag(word, command_dynamic, value_flags) {
            opaque = true;
        }
        index += 1;
    }
    (words, indefinite)
}
