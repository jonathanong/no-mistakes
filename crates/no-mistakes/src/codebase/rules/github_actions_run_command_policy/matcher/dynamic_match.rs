use super::is_dynamic_word;
use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(super) fn could_be_value_flag(
    word: &Word,
    command_dynamic: bool,
    value_flags: &[String],
) -> bool {
    value_flags
        .iter()
        .any(|flag| dynamic_word_could_match_literal(word, command_dynamic, flag))
}

pub(super) fn is_expanded_word(word: &Word) -> bool {
    word.bare_expansion.is_some() || word.quoted_expansion.is_some() || word.expansion_may_split
}

pub(super) fn possible_terminator(word: &Word, command_dynamic: bool) -> Option<(String, usize)> {
    dynamic_word_could_match_literal(word, command_dynamic, "--")
        .then(|| (word.text.clone(), word.line))
}

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

fn delimited_end(bytes: &[u8], mut index: usize, close: u8) -> usize {
    let mut frames = vec![(close, 0u8)];
    while index < bytes.len() && !frames.is_empty() {
        let byte = bytes[index];
        let (frame_close, quote) = frames[frames.len() - 1];
        if quote != 0 {
            if quote == b'"' && byte == b'\\' && index + 1 < bytes.len() {
                index += 2;
                continue;
            }
            if quote == b'"' {
                if let Some(nested) = substitution_close(bytes, index) {
                    frames.push((nested, 0));
                    index += 2;
                    continue;
                }
            }
            if byte == quote {
                let last = frames.len() - 1;
                frames[last].1 = 0;
            }
            index += 1;
            continue;
        }
        if byte == b'\\' && index + 1 < bytes.len() {
            index += 2;
            continue;
        }
        if byte == b'\'' || byte == b'"' {
            let last = frames.len() - 1;
            frames[last].1 = byte;
            index += 1;
            continue;
        }
        if let Some(nested) = substitution_close(bytes, index) {
            frames.push((nested, 0));
            index += 2;
            continue;
        }
        if byte == frame_close {
            frames.pop();
            index += 1;
            continue;
        }
        if byte == opener_for(frame_close) {
            frames.push((frame_close, 0));
            index += 1;
            continue;
        }
        index += 1;
    }
    index
}

fn substitution_close(bytes: &[u8], index: usize) -> Option<u8> {
    if bytes.get(index) != Some(&b'$') {
        return None;
    }
    match bytes.get(index + 1) {
        Some(b'(') => Some(b')'),
        Some(b'{') => Some(b'}'),
        _ => None,
    }
}

fn opener_for(close: u8) -> u8 {
    if close == b')' {
        b'('
    } else {
        b'{'
    }
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
        return Some(delimited_end(bytes, start + 2, close));
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
