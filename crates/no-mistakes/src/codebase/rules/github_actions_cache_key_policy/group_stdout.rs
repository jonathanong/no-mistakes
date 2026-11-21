use super::super::writes_output;
use crate::fx::{fx_set, FxHashSet};

pub(super) enum Stdout {
    Inherit,
    Prove,
    Leave,
}

pub(super) fn stdout_leaves(line: &str) -> bool {
    !matches!(stdout_binding(line, &fx_set()), Stdout::Inherit)
}

pub(super) fn stdout_binding(line: &str, outputs: &FxHashSet<String>) -> Stdout {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut quote = None;
    let mut arith = 0i32;
    let mut index = 0;
    let mut effect = Stdout::Inherit;
    while index < chars.len() {
        let (_, ch) = chars[index];
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if super::body::starts(&chars, index, &['$', '(', '(']) {
            arith += 1;
            index += 3;
            continue;
        }
        if arith > 0 && super::body::starts(&chars, index, &[')', ')']) {
            arith -= 1;
            index += 2;
            continue;
        }
        if arith > 0 {
            index += 1;
            continue;
        }
        if ch == '>' && index > 0 && chars[index - 1].1 == '>' {
            index += 1;
            continue;
        }
        if ch == '>' && fd_before(&chars, index) == 1 {
            if chars.get(index + 1).is_some_and(|(_, next)| *next == '&') {
                if dup_leaves(&chars, index) {
                    effect = Stdout::Leave;
                }
            } else {
                let (word, next) = redirect_word(line, &chars, index);
                effect = if !word.is_empty() && writes_output(&format!(">{word}"), outputs) {
                    Stdout::Prove
                } else {
                    Stdout::Leave
                };
                index = next.max(index + 1);
                continue;
            }
        }
        index += 1;
    }
    effect
}

fn redirect_word(line: &str, chars: &[(usize, char)], gt: usize) -> (String, usize) {
    let mut at = gt + 1;
    if chars.get(at).is_some_and(|(_, ch)| *ch == '>') {
        at += 1;
    }
    while chars.get(at).is_some_and(|(_, ch)| ch.is_whitespace()) {
        at += 1;
    }
    let start = at;
    let mut quote = None;
    while let Some((_, ch)) = chars.get(at) {
        if let Some(open) = quote {
            if *ch == open {
                quote = None;
            }
            at += 1;
            continue;
        }
        if *ch == '"' || *ch == '\'' {
            quote = Some(*ch);
            at += 1;
            continue;
        }
        if ch.is_whitespace() || boundary(*ch) {
            break;
        }
        at += 1;
    }
    let byte = chars
        .get(start)
        .map(|(offset, _)| *offset)
        .unwrap_or(line.len());
    let end = chars
        .get(at)
        .map(|(offset, _)| *offset)
        .unwrap_or(line.len());
    (line[byte..end].to_string(), at)
}

fn dup_leaves(chars: &[(usize, char)], gt: usize) -> bool {
    if !chars.get(gt + 1).is_some_and(|(_, ch)| *ch == '&') {
        return true;
    }
    fd_number(chars, gt + 2) != Some(1)
}

fn fd_before(chars: &[(usize, char)], gt: usize) -> u32 {
    let mut index = gt;
    while index > 0 && chars[index - 1].1.is_ascii_digit() {
        index -= 1;
    }
    if index == gt || (index > 0 && !boundary(chars[index - 1].1)) {
        return 1;
    }
    let text: String = chars[index..gt].iter().map(|(_, ch)| *ch).collect();
    text.parse().unwrap_or(1)
}

fn fd_number(chars: &[(usize, char)], mut at: usize) -> Option<u32> {
    while chars.get(at).is_some_and(|(_, ch)| ch.is_whitespace()) {
        at += 1;
    }
    let start = at;
    if !chars.get(at).is_some_and(|(_, ch)| ch.is_ascii_digit()) {
        return None;
    }
    while chars.get(at).is_some_and(|(_, ch)| ch.is_ascii_digit()) {
        at += 1;
    }
    if chars.get(at).is_some_and(|(_, ch)| !boundary(*ch)) {
        return None;
    }
    let text: String = chars[start..at].iter().map(|(_, ch)| *ch).collect();
    text.parse().ok()
}

fn boundary(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '>' | '<' | '&' | '|' | ';' | '(' | ')')
}
