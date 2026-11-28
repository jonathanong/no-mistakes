use super::args::ArgMatch;
use super::dynamic_match::{
    dynamic_word_could_match_literal, is_expanded_word, possible_terminator,
};
use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(super) fn match_token_sequence_skipping(
    words: &[&Word],
    tokens: &[String],
    command_dynamic: bool,
    consumed_starts: &[bool],
) -> ArgMatch {
    match_token_sequence_masked(words, tokens, command_dynamic, consumed_starts, &[])
}

pub(super) fn match_token_sequence_masked(
    words: &[&Word],
    tokens: &[String],
    command_dynamic: bool,
    consumed_starts: &[bool],
    indefinite: &[bool],
) -> ArgMatch {
    let mut dynamic_hit = None;
    let token_len = tokens.len();
    if token_len == 0 || words.len() < token_len {
        return ArgMatch::NotMatched;
    }
    for (start, window) in words.windows(token_len).enumerate() {
        if consumed_starts.get(start).copied().unwrap_or(false) {
            continue;
        }
        match match_token_window(window, tokens, command_dynamic) {
            ArgMatch::Matched => {
                let indefinite_window = window_indefinite(indefinite, start, token_len);
                let terminator = earlier_terminator(words, start, consumed_starts, command_dynamic);
                if indefinite_window {
                    dynamic_hit
                        .get_or_insert_with(|| (words[start].text.clone(), words[start].line));
                } else if let Some(hit) = terminator {
                    dynamic_hit.get_or_insert(hit);
                } else {
                    return ArgMatch::Matched;
                }
            }
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

fn earlier_terminator(
    words: &[&Word],
    end: usize,
    consumed: &[bool],
    command_dynamic: bool,
) -> Option<(String, usize)> {
    words[..end].iter().enumerate().find_map(|(index, word)| {
        if consumed.get(index).copied().unwrap_or(false) {
            return None;
        }
        possible_terminator(word, command_dynamic)
    })
}

fn window_indefinite(indefinite: &[bool], start: usize, len: usize) -> bool {
    indefinite
        .get(start..start + len)
        .is_some_and(|flags| flags.contains(&true))
}

fn match_token_window(window: &[&Word], tokens: &[String], command_dynamic: bool) -> ArgMatch {
    let mut dynamic_hit = None;
    for (word, expected) in window.iter().zip(tokens) {
        if word.text == *expected && !is_expanded_word(word) {
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
