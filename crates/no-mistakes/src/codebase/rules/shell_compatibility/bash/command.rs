use super::super::lex::{command_words, identifier, Word};
use super::super::read_targets::targets;
use super::assign::{array_assignment, forget_unset, remember_array};
use super::case_unset::CaseUnsetCtx;
use super::condition;
use super::guard::{flag, nounset};
use super::state::{ArrayState, State};

pub(super) fn remember_command<'a>(
    words: &[Word<'a>],
    in_function: bool,
    callback_known: bool,
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
        if read.callback.is_some() && !callback_known {
            state.record_mapfile_callback();
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
        "declare" | "typeset" | "local" => {
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
                        state.redeclare_local(word.raw, array);
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
