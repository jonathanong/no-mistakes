use super::args::{takes_value, ArgMatch};
use super::dynamic_match::{
    dynamic_str_could_match_literal, dynamic_word_could_match_literal,
    value_is_dynamically_emptiable,
};
use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(super) fn dynamic_inline_pair(
    word: &Word,
    command_dynamic: bool,
    flag: &str,
    value: &str,
    value_flags: &[String],
) -> bool {
    if dynamic_word_could_match_literal(word, command_dynamic, &format!("{flag}={value}")) {
        return true;
    }
    is_short_flag(flag)
        && takes_value(flag, value_flags)
        && dynamic_word_could_match_literal(word, command_dynamic, &format!("{flag}{value}"))
}

pub(super) fn match_single_flag(
    word: &Word,
    next: Option<&Word>,
    target: &str,
    value_flags: &[String],
    require_value: bool,
    command_dynamic: bool,
) -> ArgMatch {
    let needs_val = require_value && takes_value(target, value_flags);
    if word.text == target {
        if !needs_val {
            return ArgMatch::Matched;
        }
        return match next {
            None => ArgMatch::NotMatched,
            Some(val) if val.text.is_empty() => ArgMatch::NotMatched,
            Some(val) if value_is_dynamically_emptiable(&val.text, val, command_dynamic) => {
                ArgMatch::DynamicValue {
                    token: val.text.clone(),
                    line: val.line,
                }
            }
            Some(_) => ArgMatch::Matched,
        };
    }
    if target.starts_with("--") && target.len() > 2 {
        if let Some(val) = word
            .text
            .strip_prefix(target)
            .and_then(|rest| rest.strip_prefix('='))
        {
            return judge_inline_value(val, word, needs_val, command_dynamic);
        }
    }
    if is_short_flag(target) {
        if let Some(rest) = word.text.strip_prefix(target) {
            if let Some(val) = rest.strip_prefix('=') {
                return judge_inline_value(val, word, needs_val, command_dynamic);
            }
            if !rest.is_empty() && takes_value(target, value_flags) {
                return judge_inline_value(rest, word, needs_val, command_dynamic);
            }
        }
    }
    if dynamic_word_could_match_literal(word, command_dynamic, target) {
        return ArgMatch::DynamicValue {
            token: word.text.clone(),
            line: word.line,
        };
    }
    ArgMatch::NotMatched
}

fn judge_inline_value(val: &str, word: &Word, needs_val: bool, command_dynamic: bool) -> ArgMatch {
    if !needs_val {
        return ArgMatch::Matched;
    }
    if val.is_empty() {
        return ArgMatch::NotMatched;
    }
    if value_is_dynamically_emptiable(val, word, command_dynamic) {
        return ArgMatch::DynamicValue {
            token: word.text.clone(),
            line: word.line,
        };
    }
    ArgMatch::Matched
}

pub(super) fn match_flag_pair(
    word: &Word,
    next: Option<&Word>,
    flag: &str,
    value: &str,
    value_flags: &[String],
    command_dynamic: bool,
) -> ArgMatch {
    if word.text == flag {
        if let Some(val) = next {
            if val.text == value {
                return ArgMatch::Matched;
            }
            if dynamic_word_could_match_literal(val, command_dynamic, value) {
                return ArgMatch::DynamicValue {
                    token: val.text.clone(),
                    line: val.line,
                };
            }
        }
        return ArgMatch::NotMatched;
    }
    if let Some(rest) = word.text.strip_prefix(flag) {
        if let Some(inline) = rest.strip_prefix('=') {
            if inline == value {
                return ArgMatch::Matched;
            }
            if dynamic_str_could_match_literal(inline, word, command_dynamic, value) {
                return ArgMatch::DynamicValue {
                    token: word.text.clone(),
                    line: word.line,
                };
            }
        } else if is_short_flag(flag) && takes_value(flag, value_flags) && !rest.is_empty() {
            if rest == value {
                return ArgMatch::Matched;
            }
            if dynamic_str_could_match_literal(rest, word, command_dynamic, value) {
                return ArgMatch::DynamicValue {
                    token: word.text.clone(),
                    line: word.line,
                };
            }
        }
    }
    if dynamic_pair(word, next, flag, value, value_flags, command_dynamic) {
        return ArgMatch::DynamicValue {
            token: word.text.clone(),
            line: word.line,
        };
    }
    ArgMatch::NotMatched
}

fn dynamic_pair(
    word: &Word,
    next: Option<&Word>,
    flag: &str,
    value: &str,
    value_flags: &[String],
    command_dynamic: bool,
) -> bool {
    let name_matches = dynamic_word_could_match_literal(word, command_dynamic, flag);
    let next_matches = next.is_some_and(|val| {
        val.text == value || dynamic_word_could_match_literal(val, command_dynamic, value)
    });
    (name_matches && next_matches)
        || dynamic_inline_pair(word, command_dynamic, flag, value, value_flags)
}

fn is_short_flag(text: &str) -> bool {
    text.len() == 2 && text.starts_with('-') && !text.starts_with("--")
}
