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
    while index < lines.len() && !span_close(lines[index], paren) {
        let line = lines[index].trim();
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
    let (fate, rest) = if index < lines.len() {
        closer_binding(lines[index], &entry)
    } else {
        (Fate::Escape, "")
    };
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

fn closer_binding<'a>(line: &'a str, entry: &FxHashSet<String>) -> (Fate, &'a str) {
    let rest = after_first(line);
    let (redirect, after) = split_head(rest);
    let fate = if emit::piped_away(rest) {
        Fate::Discard
    } else {
        emit::fate_of(redirect, entry)
    };
    (fate, after)
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

fn span_close(line: &str, paren: bool) -> bool {
    if paren {
        line.trim().starts_with(')')
    } else {
        brace_close(line)
    }
}

#[cfg(test)]
#[path = "group_span_tests.rs"]
mod tests;
