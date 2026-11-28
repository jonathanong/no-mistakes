use super::args::ArgMatch;
use super::dynamic_match::dynamic_word_could_match_literal;
use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(super) fn match_token_sequence(
    words: &[&Word],
    tokens: &[String],
    command_dynamic: bool,
) -> ArgMatch {
    match_token_sequence_skipping(words, tokens, command_dynamic, &[])
}

pub(super) fn match_token_sequence_skipping(
    words: &[&Word],
    tokens: &[String],
    command_dynamic: bool,
    consumed_starts: &[bool],
) -> ArgMatch {
    let mut dynamic_hit = None;
    for (start, window) in words.windows(tokens.len()).enumerate() {
        if consumed_starts.get(start).copied().unwrap_or(false) {
            continue;
        }
        match match_token_window(window, tokens, command_dynamic) {
            ArgMatch::Matched => return ArgMatch::Matched,
            ArgMatch::DynamicValue { token, line } => {
                dynamic_hit.get_or_insert((token, line));
            }
            ArgMatch::NotMatched => {}
        }
    }
    match dynamic_hit {
        Some((token, line)) => ArgMatch::DynamicValue { token, line },
        None => ArgMatch::NotMatched,
    }
}

fn match_token_window(window: &[&Word], tokens: &[String], command_dynamic: bool) -> ArgMatch {
    let mut dynamic_hit = None;
    for (word, expected) in window.iter().zip(tokens) {
        if word.text == *expected {
            continue;
        }
        if dynamic_word_could_match_literal(word, command_dynamic, expected) {
            dynamic_hit.get_or_insert_with(|| (word.text.clone(), word.line));
            continue;
        }
        return ArgMatch::NotMatched;
    }
    match dynamic_hit {
        Some((token, line)) => ArgMatch::DynamicValue { token, line },
        None => ArgMatch::Matched,
    }
}
