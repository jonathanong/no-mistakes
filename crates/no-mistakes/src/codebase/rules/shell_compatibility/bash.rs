use super::lex::{Boundary, Lexer, Shell};
use command::remember_command;
use flow::Flow;
use state::{ArrayState, UnanalyzableReason};

mod assign;
mod branch_frame;
mod case_frame;
mod case_unset;
mod command;
mod condition;
mod flow;
mod frames;
mod guard;
mod loop_frame;
mod state;
mod write_sets;

#[cfg(test)]
mod loop_tests;
#[cfg(test)]
mod tests;

pub const RULE_ID: &str = "bash-empty-array-nounset";

#[derive(Default)]
pub(super) struct ScanResult<'a> {
    pub findings: Vec<(&'a str, usize)>,
    pub unanalyzable: Vec<(usize, String)>,
}

pub(super) fn analyze(source: &str) -> ScanResult<'_> {
    let mut flow = Flow::default();
    let mut hits = Vec::new();
    let mut commands = Vec::new();
    Lexer::new(source, Shell::Bash).commands(|words, scope, boundary| {
        commands.push(write_sets::Command {
            words,
            scope,
            boundary,
        });
    });
    let write_sets = write_sets::WriteSets::new(&commands);
    let mut index = 0;
    while index < commands.len() {
        let command = &commands[index];
        let mut ref_unanalyzable = Vec::new();
        let restart = flow.command(index, command, &write_sets, hits.len(), |state, frames| {
            let dead_arm = frames.iter().any(|frame| match frame {
                frames::Frame::Case(case) => case.arm_is_dead(),
                frames::Frame::If(branch) => branch.arm_is_dead(),
                _ => false,
            });
            if state.active && !dead_arm {
                for (name, line) in command.words.iter().flat_map(|w| w.refs.iter().copied()) {
                    match state.arrays.get(name).copied() {
                        Some(ArrayState::Empty) => hits.push((name, line)),
                        Some(ArrayState::MaybeEmpty) if state.guarded_maybe.contains(name) => {
                            ref_unanalyzable.push((
                                line,
                                format!(
                                    "array `{name}` safety could not be proved under guarding condition; guard with ${{{name}[@]+...}}"
                                ),
                            ));
                        }
                        Some(ArrayState::MaybeEmpty) => hits.push((name, line)),
                        _ => {
                            if let Some(&reason) = state.unanalyzable.get(name) {
                                let cause = match reason {
                                    UnanalyzableReason::LoopWidening => "after loop widening",
                                    UnanalyzableReason::DynamicUnset => "after dynamic `unset`",
                                    UnanalyzableReason::MapfileCallback => {
                                        "after `mapfile -C` callback"
                                    }
                                };
                                ref_unanalyzable.push((
                                    line,
                                    format!(
                                        "array `{name}` safety could not be proved {cause}; guard with ${{{name}[@]+...}}"
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
            if command.boundary == Boundary::CasePattern {
                condition::vars::collect_arithmetic_writes(&command.words, |name| {
                    state.touch(name);
                    if let Some(frames::Frame::Case(case)) = frames.last_mut() {
                        case.touch_pattern_write(name);
                    }
                });
            } else {
                remember_command(
                    &command.words,
                    command.scope > 0,
                    write_sets.call_is_callback(index),
                    state,
                    frames,
                );
            }
        });
        flow.unanalyzable.extend(ref_unanalyzable);
        if let Some((next_index, truncate_hits, truncate_unanalyzable)) = restart {
            hits.truncate(truncate_hits);
            flow.unanalyzable.truncate(truncate_unanalyzable);
            index = next_index;
        } else {
            index += 1;
        }
    }
    ScanResult {
        findings: hits,
        unanalyzable: flow.unanalyzable,
    }
}
