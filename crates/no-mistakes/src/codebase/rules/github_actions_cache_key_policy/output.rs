use super::output_syntax::{
    brace_close, brace_open, command_assign, heredoc_end, heredoc_start, inline_assign,
    shell_assignment, strip_subst, unquote, var_ref,
};
use super::text::{command_hashes_lockfile, shell_line};
use crate::fx::{fx_set, FxHashSet};

pub(super) struct Write {
    pub(super) name: String,
    pub(super) value: String,
    pub(super) expand: bool,
}

pub(super) enum Event {
    Assign(String, String),
    Write(Write),
}

pub(super) fn proven_output_names(run: &str, lockfile: &str) -> Vec<String> {
    let mut vars = fx_set();
    let mut names = Vec::new();
    for event in events(run) {
        match event {
            Event::Assign(name, rhs) => {
                let keep = rhs_hashes(&rhs, &vars, lockfile)
                    || (vars.contains(&name) && super::expand::same_name_expansion(&rhs, &name));
                if keep {
                    vars.insert(name);
                } else {
                    vars.remove(&name);
                }
            }
            Event::Write(write) => {
                if write.expand
                    && expression_hashes(&write.value, &vars, lockfile)
                    && !names.contains(&write.name)
                {
                    names.push(write.name);
                }
            }
        }
    }
    names
}

fn rhs_hashes(rhs: &str, vars: &FxHashSet<String>, lockfile: &str) -> bool {
    if let Some(cmd) = strip_subst(rhs) {
        return command_hashes_lockfile(cmd, lockfile);
    }
    expression_hashes(rhs, vars, lockfile)
}

fn expression_hashes(value: &str, vars: &FxHashSet<String>, lockfile: &str) -> bool {
    let trimmed = value.trim().strip_suffix("\\n").unwrap_or(value.trim());
    if super::expand::wholly_single_quoted(trimmed) {
        return false;
    }
    let value = unquote(trimmed);
    if let Some(name) = var_ref(value) {
        return vars.contains(name);
    }
    strip_subst(value).is_some_and(|cmd| command_hashes_lockfile(cmd, lockfile))
}

pub(super) fn events(run: &str) -> Vec<Event> {
    let lines: Vec<&str> = run.lines().map(shell_line).collect();
    let mut events = Vec::new();
    let mut outputs = fx_set();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if let Some((delim, strip, quoted)) = heredoc_start(line) {
            let active = writes_output(line, &outputs);
            index += 1;
            while index < lines.len() && !heredoc_end(lines[index], &delim, strip) {
                if active {
                    if let Some((name, value)) = inline_assign(lines[index]) {
                        events.push(Event::Write(Write {
                            name,
                            value,
                            expand: !quoted,
                        }));
                    }
                }
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            continue;
        }
        if brace_open(line) {
            index = brace_events(&lines, index, &mut events, &mut outputs);
            continue;
        }
        if let Some((name, rhs)) = shell_assignment(line) {
            remember_assign(&mut events, &mut outputs, name, rhs);
        } else if writes_output(line, &outputs) {
            if let Some((name, value)) = command_assign(line) {
                events.push(Event::Write(Write {
                    name,
                    value,
                    expand: true,
                }));
            }
        }
        index += 1;
    }
    events
}

fn remember_assign(
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
    name: String,
    rhs: String,
) {
    if output_path(&rhs, outputs) {
        outputs.insert(name.clone());
    } else {
        outputs.remove(&name);
    }
    events.push(Event::Assign(name, rhs));
}

fn output_path(rhs: &str, outputs: &FxHashSet<String>) -> bool {
    let rhs = unquote(rhs.trim().trim_end_matches(';').trim());
    match var_ref(rhs) {
        Some("GITHUB_OUTPUT") => true,
        Some(name) => outputs.contains(name),
        None => false,
    }
}

fn writes_output(line: &str, outputs: &FxHashSet<String>) -> bool {
    super::shell_cmd::redirect_targets(line)
        .iter()
        .any(|target| output_path(target, outputs))
}

fn brace_events(
    lines: &[&str],
    start: usize,
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
) -> usize {
    let mut armed = writes_output(lines[start], outputs);
    let mut pending = Vec::new();
    let mut index = start + 1;
    while index < lines.len() && !brace_close(lines[index]) {
        let line = lines[index].trim();
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
