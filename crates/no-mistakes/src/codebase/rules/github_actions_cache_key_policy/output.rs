use super::output_syntax::{
    brace_close, brace_open, command_assign, heredoc_end, heredoc_start, inline_assign,
    shell_assignment, strip_subst, unquote, var_ref,
};
use super::text::{command_hashes_lockfile, shell_line};
use crate::fx::{fx_set, FxHashSet};

struct Write {
    name: String,
    value: String,
    expand: bool,
}

enum Event {
    Assign(String, String),
    Write(Write),
}

#[cfg(test)]
pub(super) fn output_names(run: &str) -> Vec<String> {
    let mut names = Vec::new();
    for event in events(run) {
        let Event::Write(write) = event else {
            continue;
        };
        if !names.contains(&write.name) {
            names.push(write.name);
        }
    }
    names
}

pub(super) fn proven_output_names(run: &str, lockfile: &str) -> Vec<String> {
    let mut vars = fx_set();
    let mut names = Vec::new();
    for event in events(run) {
        match event {
            Event::Assign(name, rhs) => {
                if rhs_hashes(&rhs, &vars, lockfile) {
                    vars.insert(name);
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
    let value = unquote(value.trim().strip_suffix("\\n").unwrap_or(value.trim()));
    if let Some(name) = var_ref(value) {
        return vars.contains(name);
    }
    strip_subst(value).is_some_and(|cmd| command_hashes_lockfile(cmd, lockfile))
}

fn events(run: &str) -> Vec<Event> {
    let lines: Vec<&str> = run.lines().map(shell_line).collect();
    let mut events = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if let Some((delim, strip, quoted)) = heredoc_start(line) {
            let active = line.contains("GITHUB_OUTPUT");
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
            index = brace_events(&lines, index, &mut events);
            continue;
        }
        if line.contains("GITHUB_OUTPUT") {
            if let Some((name, value)) = command_assign(line) {
                events.push(Event::Write(Write {
                    name,
                    value,
                    expand: true,
                }));
            }
        } else if let Some((name, rhs)) = shell_assignment(line) {
            events.push(Event::Assign(name, rhs));
        }
        index += 1;
    }
    events
}

fn brace_events(lines: &[&str], start: usize, events: &mut Vec<Event>) -> usize {
    let mut armed = lines[start].contains("GITHUB_OUTPUT");
    let mut pending = Vec::new();
    let mut index = start + 1;
    while index < lines.len() && !brace_close(lines[index]) {
        let line = lines[index].trim();
        if let Some((name, rhs)) = shell_assignment(line) {
            events.push(Event::Assign(name, rhs));
        } else if let Some((name, value)) = command_assign(line) {
            if line.contains("GITHUB_OUTPUT") {
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
    if index < lines.len() && lines[index].contains("GITHUB_OUTPUT") {
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
