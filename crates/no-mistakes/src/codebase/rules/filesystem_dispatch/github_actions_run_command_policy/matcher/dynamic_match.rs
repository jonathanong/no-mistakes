use super::is_dynamic_word;
use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(super) fn dynamic_word_could_match_literal(
    word: &Word,
    command_dynamic: bool,
    target: &str,
) -> bool {
    dynamic_str_could_match_literal(&word.text, word, command_dynamic, target)
}

pub(super) fn dynamic_str_could_match_literal(
    value: &str,
    word: &Word,
    command_dynamic: bool,
    target: &str,
) -> bool {
    if !is_dynamic_word(word, command_dynamic) {
        return false;
    }
    let parts = split_dynamic_segments(value);
    if parts.len() <= 1 {
        return false;
    }
    let Some(rest) = target.strip_prefix(parts[0]) else {
        return false;
    };
    let Some(mut rest) = rest.strip_suffix(parts[parts.len() - 1]) else {
        return false;
    };
    for mid in &parts[1..parts.len() - 1] {
        let Some(pos) = rest.find(mid) else {
            return false;
        };
        rest = &rest[pos + mid.len()..];
    }
    true
}

pub(super) fn value_is_dynamically_emptiable(
    value: &str,
    word: &Word,
    command_dynamic: bool,
) -> bool {
    if !is_dynamic_word(word, command_dynamic) {
        return false;
    }
    let parts = split_dynamic_segments(value);
    parts.len() > 1 && parts.iter().all(|part| part.is_empty())
}

fn split_dynamic_segments(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut seg_start = 0;
    let mut i = 0;
    while i < text.len() {
        if let Some(end) = dynamic_span_end(text, i) {
            parts.push(&text[seg_start..i]);
            i = end;
            seg_start = i;
        } else {
            i += 1;
        }
    }
    parts.push(&text[seg_start..]);
    parts
}

fn dynamic_span_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes[start] == b'`' {
        return Some(
            text[start + 1..]
                .find('`')
                .map_or(bytes.len(), |offset| start + 2 + offset),
        );
    }
    if bytes[start] != b'$' {
        return None;
    }
    let next = *bytes.get(start + 1)?;
    if next == b'{' && bytes.get(start + 2) == Some(&b'{') {
        return Some(
            text[start + 3..]
                .find("}}")
                .map_or(bytes.len(), |offset| start + 5 + offset),
        );
    }
    if next == b'{' || next == b'(' {
        let close = if next == b'{' { b'}' } else { b')' };
        let mut depth = 1usize;
        let mut i = start + 2;
        while i < bytes.len() && depth > 0 {
            depth = depth + usize::from(bytes[i] == next) - usize::from(bytes[i] == close);
            i += 1;
        }
        return Some(i);
    }
    if next.is_ascii_digit() {
        return Some(start + 2);
    }
    if next.is_ascii_alphanumeric() || next == b'_' {
        let len = bytes[start + 2..]
            .iter()
            .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_')
            .count();
        return Some(start + 2 + len);
    }
    matches!(next, b'@' | b'*' | b'#' | b'?' | b'-' | b'$' | b'!').then_some(start + 2)
}
