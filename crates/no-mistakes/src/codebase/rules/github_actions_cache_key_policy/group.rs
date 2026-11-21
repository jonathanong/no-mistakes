use super::super::output_syntax::{brace_close, brace_open, command_assign, shell_assignment};
use super::{remember_assign, writes_output, Event, Write};
use crate::fx::FxHashSet;

#[path = "group_body.rs"]
mod body;
#[path = "group_stdout.rs"]
mod stdout;

pub(super) fn consume(
    lines: &[&str],
    index: usize,
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
) -> Option<usize> {
    let line = lines[index].trim();
    if body::closed_group(line).is_some() {
        record_closed(events, outputs, line);
        return Some(index + 1);
    }
    if brace_open(line) || line.starts_with('(') {
        return Some(span_events(lines, index, events, outputs));
    }
    None
}

fn record_closed(events: &mut Vec<Event>, outputs: &mut FxHashSet<String>, line: &str) {
    let Some((body, suffix)) = body::closed_group(line) else {
        return;
    };
    for (command, inherit) in body::inheriting(body) {
        let applied = if inherit { suffix } else { "" };
        record_command(events, outputs, command, applied);
    }
}

fn record_command(
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
    command: &str,
    suffix: &str,
) {
    if let Some((name, rhs)) = shell_assignment(command) {
        remember_assign(events, outputs, name, rhs);
        return;
    }
    let owned;
    let line = if suffix.is_empty() || stdout::stdout_leaves(command) {
        command
    } else {
        owned = format!("{command} {suffix}");
        owned.as_str()
    };
    if !writes_output(line, outputs) {
        return;
    }
    if let Some((name, value)) = command_assign(line) {
        events.push(Event::Write(Write {
            name,
            value,
            expand: true,
        }));
    }
}

fn span_events(
    lines: &[&str],
    start: usize,
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
) -> usize {
    let paren = lines[start].trim().starts_with('(');
    let mut armed = writes_output(lines[start], outputs);
    let mut pending = Vec::new();
    let mut index = start + 1;
    while index < lines.len() && !span_close(lines[index], paren) {
        let line = lines[index].trim();
        if let Some((body, suffix)) = body::closed_group(line) {
            if suffix.is_empty() {
                for (command, inherit) in body::inheriting(body) {
                    if inherit {
                        push_span(events, outputs, &mut pending, command);
                    } else if let Some((name, rhs)) = shell_assignment(command) {
                        remember_assign(events, outputs, name, rhs);
                    }
                }
            } else {
                record_closed(events, outputs, line);
            }
        } else {
            push_span(events, outputs, &mut pending, line);
        }
        index += 1;
    }
    if index < lines.len() && writes_output(lines[index], outputs) {
        armed = true;
    }
    if armed {
        events.extend(pending.into_iter().map(Event::Write));
    }
    if index < lines.len() {
        index + 1
    } else {
        index
    }
}

fn push_span(
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
    pending: &mut Vec<Write>,
    line: &str,
) {
    if let Some((name, rhs)) = shell_assignment(line) {
        remember_assign(events, outputs, name, rhs);
    } else if let Some((name, value)) = command_assign(line) {
        if writes_output(line, outputs) {
            events.push(Event::Write(Write {
                name,
                value,
                expand: true,
            }));
        } else {
            pending.push(Write {
                name,
                value,
                expand: true,
            });
        }
    }
}

fn span_close(line: &str, paren: bool) -> bool {
    let line = line.trim();
    if paren {
        line == ")" || line.starts_with(") ") || line.starts_with(")>>") || line.starts_with(") >>")
    } else {
        brace_close(line)
    }
}
