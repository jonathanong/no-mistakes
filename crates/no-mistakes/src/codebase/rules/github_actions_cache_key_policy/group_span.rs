use super::super::super::output_syntax::{brace_close, brace_open};
use super::super::super::shell_cmd::split_head;
use super::super::Event;
use super::body;
use super::emit::{self, Fate, Frame, Pending};
use crate::fx::FxHashSet;

pub(super) fn scan(
    lines: &[&str],
    index: usize,
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
) -> Option<usize> {
    let line = lines[index].trim();
    let mut held = Vec::new();
    let next = if body::closed_group(line).is_some() {
        emit::record_closed(&mut held, outputs, line, false);
        index + 1
    } else if opens(line) {
        span_events(lines, index, &mut held, outputs, false)
    } else {
        return None;
    };
    commit(events, held);
    Some(next)
}

pub(super) fn commit(events: &mut Vec<Event>, held: Vec<Pending>) {
    for item in held {
        if let Pending::Event(event) = item {
            events.push(event);
        }
    }
}

fn span_events(
    lines: &[&str],
    start: usize,
    sink: &mut Vec<Pending>,
    outputs: &mut FxHashSet<String>,
    nested: bool,
) -> usize {
    let opener = lines[start].trim();
    let paren = opener.starts_with('(');
    let entry = outputs.clone();
    let mut pending = Vec::new();
    let opener_body = after_first(opener);
    if !opener_body.is_empty() {
        emit::record_text(&mut pending, outputs, opener_body, true);
    }
    let mut index = start + 1;
    let mut fate = Fate::Escape;
    let mut rest = "";
    while index < lines.len() {
        let line = lines[index].trim();
        if let Some((before, after)) = split_closer(line, paren) {
            if !before.is_empty() {
                emit::record_text(&mut pending, outputs, before, true);
            }
            (fate, rest) = bind_suffix(after, &entry);
            break;
        }
        if body::closed_group(line).is_some() {
            emit::record_closed(&mut pending, outputs, line, true);
        } else if opens(line) {
            index = span_events(lines, index, &mut pending, outputs, true);
            continue;
        } else {
            emit::record_text(&mut pending, outputs, line, true);
        }
        index += 1;
    }
    emit::emit(
        sink,
        pending,
        outputs,
        &entry,
        Frame {
            paren,
            fate,
            nested,
        },
    );
    if !rest.is_empty() {
        emit::record_text(sink, outputs, rest, nested);
    }
    if index < lines.len() {
        index + 1
    } else {
        index
    }
}

fn bind_suffix<'a>(suffix: &'a str, entry: &FxHashSet<String>) -> (Fate, &'a str) {
    let (redirect, after) = split_head(suffix);
    let fate = if emit::piped_away(suffix) {
        Fate::Discard
    } else {
        emit::fate_of(redirect, entry)
    };
    (fate, after)
}

fn split_closer(line: &str, paren: bool) -> Option<(&str, &str)> {
    let line = line.trim();
    if paren && line.starts_with(')') {
        return Some(("", after_first(line)));
    }
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut quote = None;
    let mut escape = false;
    let mut parens = 0i32;
    let mut braces = 0i32;
    for (index, (at, ch)) in chars.iter().enumerate() {
        if let Some(open) = quote {
            if escape {
                escape = false;
            } else if *ch == '\\' && open == '"' {
                escape = true;
            } else if *ch == open {
                quote = None;
            }
            continue;
        }
        if *ch == '"' || *ch == '\'' {
            quote = Some(*ch);
            escape = false;
            continue;
        }
        if *ch == '#' && (index == 0 || chars[index - 1].1.is_whitespace()) {
            break;
        }
        match *ch {
            '{' => braces += 1,
            '}' if braces > 0 => braces -= 1,
            '}' if !paren
                && braces == 0
                && parens == 0
                && brace_close(&line[*at..])
                && brace_token(&chars, index) =>
            {
                return Some((line[..*at].trim(), line[*at + ch.len_utf8()..].trim()));
            }
            '(' => parens += 1,
            ')' if parens > 0 => parens -= 1,
            ')' if paren && parens == 0 && braces == 0 => {
                return Some((line[..*at].trim(), line[*at + ch.len_utf8()..].trim()));
            }
            _ => {}
        }
    }
    None
}

fn brace_token(chars: &[(usize, char)], index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 && chars[cursor - 1].1.is_whitespace() {
        cursor -= 1;
    }
    if cursor == 0 {
        return true;
    }
    match chars[cursor - 1].1 {
        ';' => true,
        '&' => !matches!(
            cursor.checked_sub(2).map(|prev| chars[prev].1),
            Some('&' | '>' | '<' | '|')
        ),
        _ => false,
    }
}

pub(super) fn after_first(line: &str) -> &str {
    let line = line.trim();
    match line.chars().next() {
        Some(ch) => line[ch.len_utf8()..].trim(),
        None => "",
    }
}

fn opens(line: &str) -> bool {
    brace_open(line) || line.starts_with('(')
}

#[cfg(test)]
#[path = "group_span_tests.rs"]
mod tests;
