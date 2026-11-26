use super::lex::{command_words, identifier, Boundary, Lexer, Shell, Word};
use super::read_targets::targets;
use assign::{array_assignment, forget_unset, remember_array};
use case_unset::CaseUnsetCtx;
use flow::Flow;
use guard::{flag, nounset};
use state::{ArrayState, State, UnanalyzableReason};

mod assign;
mod branch_frame;
mod case_frame;
mod case_unset;
mod condition;
mod flow;
mod frames;
mod guard;
mod loop_frame;
mod state;
mod write_sets;

#[cfg(test)]
mod loop_budget_tests;
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
                remember_command(&command.words, command.scope > 0, state, frames);
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

fn remember_command<'a>(
    words: &[Word<'a>],
    in_function: bool,
    state: &mut State,
    case_ctx: &mut dyn CaseUnsetCtx,
) {
    condition::vars::collect_arithmetic_writes(words, |name| {
        state.touch(name);
        case_ctx.invalidate(name);
    });
    if let Some(read) = targets(words) {
        for name in read.names {
            let nonempty = read.keep && state.arrays.get(&name) == Some(&ArrayState::Nonempty);
            state.write_array(&name, nonempty, false, read.keep);
            if !nonempty {
                state.set_known(&name, ArrayState::MaybeEmpty);
            }
            case_ctx.invalidate(&name);
        }
        for scalar in read.scalars {
            state.touch(&scalar);
            if state.arrays.get(&scalar) == Some(&ArrayState::Empty) {
                state.set_known(&scalar, ArrayState::MaybeEmpty);
            }
            case_ctx.invalidate(&scalar);
        }
        return;
    }
    let mut command = command_words(words).peekable();
    let Some(first) = command.peek() else {
        return;
    };
    match first.raw {
        "set" => {
            if let Some(active) = nounset(command.skip(1)) {
                state.set_nounset(active);
            }
        }
        "shopt" => {
            if command.skip(1).any(|word| {
                flag(word.raw).is_some_and(|(on, f)| on && (f.contains('s') || f.contains('u')))
            }) {
                state.conditional.clear();
            }
        }
        "unset" => forget_unset(command.skip(1), state, case_ctx),
        "declare" | "local" => {
            let mut array = false;
            let mut global = false;
            let mut display = false;
            let local_kw = first.raw == "local";
            command.next();
            while let Some((enable, flags)) = command.peek().and_then(|word| flag(word.raw)) {
                if flags.contains('a') {
                    array = enable;
                }
                if flags.contains('g') {
                    global = enable;
                }
                display |= enable && flags.contains(['p', 'f', 'F']);
                command.next();
            }
            if display {
                return;
            }
            let is_local = local_kw || (in_function && !global);
            command.next_if(|word| word.raw == "--");
            for word in command {
                if identifier(word.raw) {
                    if is_local {
                        let empty = array || state.arrays.contains_key(word.raw);
                        state.declare_local(word.raw, empty);
                    } else if array && !state.arrays.contains_key(word.raw) {
                        state.write_array(word.raw, false, false, true);
                    }
                    case_ctx.invalidate(word.raw);
                }
                if let Some(assignment) = array_assignment(word.raw) {
                    remember_array(state, word, assignment, array, is_local, case_ctx);
                }
            }
        }
        _ => {
            let assignments: Option<Vec<_>> = command
                .map(|word| array_assignment(word.raw).map(|assignment| (word, assignment)))
                .collect();
            for (word, assignment) in assignments.into_iter().flatten() {
                remember_array(state, word, assignment, false, false, case_ctx);
            }
        }
    }
}
