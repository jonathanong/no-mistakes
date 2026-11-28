use super::shell_syntax::{
    is_comment_start, take_comment, take_heredoc_bodies, take_heredoc_start,
};

pub(super) const ANSI_C: char = '\u{1}';

/// Walk one `$(...)` or `${...}` span.
///
/// Nested command substitutions start a fresh quote context. Parameter
/// expansions do not: bash still treats `'` as syntax inside `"${name:-O'Reilly}"`.
pub(super) fn take_expansion(
    chars: &[char],
    mut index: usize,
    word: &mut String,
    line: &mut usize,
) -> (usize, bool) {
    let close = if chars[index + 1] == '(' { ')' } else { '}' };
    word.push('$');
    word.push(chars[index + 1]);
    index += 2;
    let mut frames = vec![(close, '\0')];
    let mut heredocs = Vec::new();
    while index < chars.len() && !frames.is_empty() {
        let ch = chars[index];
        if ch == '\n' {
            *line += 1;
        }
        let (frame_close, quote) = frames[frames.len() - 1];
        if quote != '\0' {
            if matches!(quote, '"' | ANSI_C) && ch == '\\' && index + 1 < chars.len() {
                push_escaped(chars, &mut index, word, line);
                continue;
            }
            if quote == '"' {
                if let Some(nested) = substitution_close(chars, index) {
                    push_nested(&mut frames, nested, chars, &mut index, word);
                    continue;
                }
            }
            if (quote == ANSI_C && ch == '\'') || (quote != ANSI_C && ch == quote) {
                let last = frames.len() - 1;
                frames[last].1 = '\0';
            }
            word.push(ch);
            index += 1;
            continue;
        }
        if ch == '\\' && index + 1 < chars.len() {
            push_escaped(chars, &mut index, word, line);
            continue;
        }
        if frame_close == ')' {
            if is_comment_start(chars, index) {
                take_comment(chars, &mut index, word);
                continue;
            }
            if let Some(doc) = take_heredoc_start(chars, &mut index, word) {
                heredocs.push(doc);
                continue;
            }
            if ch == '\n' && !heredocs.is_empty() {
                take_heredoc_bodies(chars, &mut index, word, line, &mut heredocs);
                continue;
            }
        }
        if ch == '\'' || ch == '"' {
            let last = frames.len() - 1;
            frames[last].1 = if ch == '\'' && index > 0 && chars[index - 1] == '$' {
                ANSI_C
            } else {
                ch
            };
            word.push(ch);
            index += 1;
            continue;
        }
        if let Some(nested) = substitution_close(chars, index) {
            push_nested(&mut frames, nested, chars, &mut index, word);
            continue;
        }
        if ch == frame_close {
            frames.pop();
            word.push(ch);
            index += 1;
            continue;
        }
        if ch == opener_for(frame_close) {
            frames.push((frame_close, '\0'));
            word.push(ch);
            index += 1;
            continue;
        }
        word.push(ch);
        index += 1;
    }
    (index, frames.is_empty())
}

fn push_nested(
    frames: &mut Vec<(char, char)>,
    close: char,
    chars: &[char],
    index: &mut usize,
    word: &mut String,
) {
    frames.push((close, '\0'));
    word.push('$');
    word.push(chars[*index + 1]);
    *index += 2;
}

fn push_escaped(chars: &[char], index: &mut usize, word: &mut String, line: &mut usize) {
    word.push(chars[*index]);
    word.push(chars[*index + 1]);
    if chars[*index + 1] == '\n' {
        *line += 1;
    }
    *index += 2;
}

fn substitution_close(chars: &[char], index: usize) -> Option<char> {
    if chars.get(index) != Some(&'$') {
        return None;
    }
    match chars.get(index + 1) {
        Some('(') => Some(')'),
        Some('{') => Some('}'),
        _ => None,
    }
}

fn opener_for(close: char) -> char {
    if close == ')' { '(' } else { '{' }
}
