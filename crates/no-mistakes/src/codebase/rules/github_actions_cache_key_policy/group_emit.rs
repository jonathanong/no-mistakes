use super::super::super::output_syntax::{command_assign, shell_assignment};
use super::super::super::shell_cmd::split_head;
use super::super::{remember_assign, writes_output, Event, Write};
use super::{body, stdout};
use crate::fx::FxHashSet;

pub(super) enum Pending {
    Event(Event),
    Pass(Write),
}

pub(super) struct Frame {
    pub(super) paren: bool,
    pub(super) fate: Fate,
    pub(super) nested: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fate {
    Prove,
    Discard,
    Escape,
}

pub(super) fn record_closed(
    sink: &mut Vec<Pending>,
    outputs: &mut FxHashSet<String>,
    line: &str,
    nested: bool,
) {
    let Some((body, suffix)) = body::closed_group(line) else {
        return;
    };
    let entry = outputs.clone();
    let mut pending = Vec::new();
    record_body(&mut pending, outputs, body);
    let (redirect, rest) = split_head(suffix);
    let fate = if piped_away(suffix) {
        Fate::Discard
    } else {
        fate_of(redirect, &entry)
    };
    emit(
        sink,
        pending,
        outputs,
        &entry,
        Frame {
            paren: line.trim().starts_with('('),
            fate,
            nested,
        },
    );
    if !rest.is_empty() {
        record_text(sink, outputs, rest, nested);
    }
}

pub(super) fn record_text(
    sink: &mut Vec<Pending>,
    outputs: &mut FxHashSet<String>,
    text: &str,
    nested: bool,
) {
    if body::closed_group(text).is_some() {
        record_closed(sink, outputs, text, nested);
    } else {
        record_body(sink, outputs, text);
    }
}

fn record_body(pending: &mut Vec<Pending>, outputs: &mut FxHashSet<String>, text: &str) {
    for (command, inherit) in body::inheriting(text) {
        if body::closed_group(command).is_some() {
            record_closed(pending, outputs, command, inherit);
        } else {
            record_command(pending, outputs, command, inherit);
        }
    }
}

fn record_command(
    pending: &mut Vec<Pending>,
    outputs: &mut FxHashSet<String>,
    command: &str,
    inherit: bool,
) {
    if let Some((name, rhs)) = shell_assignment(command) {
        let mut assigned = Vec::new();
        remember_assign(&mut assigned, outputs, name, rhs);
        pending.extend(assigned.into_iter().map(Pending::Event));
        return;
    }
    if !inherit || stdout::stdout_leaves(command) {
        push_definite(pending, outputs, command);
        return;
    }
    if writes_output(command, outputs) {
        push_definite(pending, outputs, command);
        return;
    }
    if let Some((name, value)) = command_assign(command) {
        pending.push(Pending::Pass(Write {
            name,
            value,
            expand: true,
        }));
    }
}

fn push_definite(pending: &mut Vec<Pending>, outputs: &FxHashSet<String>, command: &str) {
    if !writes_output(command, outputs) {
        return;
    }
    if let Some((name, value)) = command_assign(command) {
        pending.push(Pending::Event(Event::Write(Write {
            name,
            value,
            expand: true,
        })));
    }
}

pub(super) fn emit(
    sink: &mut Vec<Pending>,
    pending: Vec<Pending>,
    outputs: &mut FxHashSet<String>,
    entry: &FxHashSet<String>,
    frame: Frame,
) {
    let keep_pass = frame.fate == Fate::Prove || (frame.fate == Fate::Escape && frame.nested);
    let kept = pending.iter().any(|item| keeps_write(item, keep_pass));
    let keep_assigns = !frame.paren || kept;
    let mut filtered = Vec::new();
    for item in pending {
        match item {
            Pending::Pass(write) if frame.fate == Fate::Prove => {
                filtered.push(Pending::Event(Event::Write(write)));
            }
            Pending::Pass(write) if keep_pass => filtered.push(Pending::Pass(write)),
            Pending::Pass(_) => {}
            Pending::Event(Event::Assign(name, rhs)) if keep_assigns => {
                filtered.push(Pending::Event(Event::Assign(name, rhs)));
            }
            Pending::Event(Event::Assign(_, _)) => {}
            Pending::Event(event) => filtered.push(Pending::Event(event)),
        }
    }
    if frame.paren {
        *outputs = entry.clone();
    }
    if frame.paren && kept {
        sink.push(Pending::Event(Event::BeginScope));
        sink.extend(filtered);
        sink.push(Pending::Event(Event::EndScope));
    } else {
        sink.extend(filtered);
    }
}

fn keeps_write(item: &Pending, keep_pass: bool) -> bool {
    match item {
        Pending::Event(Event::Write(_)) => true,
        Pending::Pass(_) => keep_pass,
        Pending::Event(_) => false,
    }
}

pub(super) fn fate_of(redirect: &str, entry: &FxHashSet<String>) -> Fate {
    if redirect.is_empty() {
        return Fate::Escape;
    }
    if writes_output(redirect, entry) {
        return Fate::Prove;
    }
    if stdout::stdout_leaves(redirect) {
        Fate::Discard
    } else {
        Fate::Escape
    }
}

pub(super) fn piped_away(suffix: &str) -> bool {
    let suffix = suffix.trim();
    suffix.starts_with('|') && !suffix.starts_with("||")
}
