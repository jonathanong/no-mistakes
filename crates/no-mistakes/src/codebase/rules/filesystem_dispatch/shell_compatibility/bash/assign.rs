use super::super::lex::{assignment, identifier, Word};
use super::case_unset::{extract_derivation, extract_single_var, CaseUnsetCtx};
use super::state::{ArrayState, State};
use crate::codebase::command_lex::words::{split_quoted_words, Whitespace};

pub(super) fn remember_array<'a>(
    state: &mut State,
    word: &Word<'a>,
    (name, append): (&'a str, bool),
    declared: bool,
    is_local: bool,
    case_ctx: &mut dyn CaseUnsetCtx,
) {
    let previous = state.arrays.get(name).copied();
    let deriv = if word.array_literal.is_none() && !append {
        word.raw
            .split_once('=')
            .filter(|(lhs, _)| identifier(lhs))
            .and_then(|(_, rhs)| {
                let d = extract_derivation(rhs)?;
                if d.0 == name {
                    state
                        .derived_from
                        .get(name)
                        .map(|(orig, ex)| (orig.clone(), *ex && d.1))
                } else {
                    Some(d)
                }
            })
    } else {
        None
    };
    if let Some(literal) = word.array_literal {
        let nonempty = literal
            || word
                .array_copies
                .iter()
                .any(|copy| state.arrays.get(*copy) == Some(&ArrayState::Nonempty))
            || (append && previous == Some(ArrayState::Nonempty));
        let maybe_empty = !nonempty
            && ((append && previous == Some(ArrayState::MaybeEmpty))
                || word
                    .array_copies
                    .iter()
                    .any(|copy| state.arrays.get(*copy) == Some(&ArrayState::MaybeEmpty)));
        state.write_array(name, nonempty, is_local, append);
        if maybe_empty {
            state.set_known(name, ArrayState::MaybeEmpty);
        }
    } else if declared || previous.is_some() {
        state.write_array(name, true, is_local, append);
    } else if is_local {
        state.declare_local(name, false);
    } else {
        state.touch(name);
    }
    case_ctx.invalidate(name);
    if let Some((src, exact)) = deriv {
        state.record_derivation(name, &src, exact);
    }
}

pub(super) fn array_assignment(word: &str) -> Option<(&str, bool)> {
    assignment(word).or_else(|| {
        let (lhs, _) = word.split_once('=')?;
        let append = lhs.ends_with('+');
        indexed_name(lhs.strip_suffix('+').unwrap_or(lhs)).map(|name| (name, append))
    })
}

pub(super) fn forget_unset<'w, 'a: 'w>(
    words: impl Iterator<Item = &'w Word<'a>>,
    state: &mut State,
    case_ctx: &mut dyn CaseUnsetCtx,
) {
    let mut end_opts = false;
    let mut dynamic = false;
    for word in words {
        let parsed = single_word(word.raw);
        if !end_opts {
            if let Some(flag) = parsed.as_deref() {
                if !has_expansion(word.raw) {
                    if flag == "-v" {
                        continue;
                    }
                    if flag == "--" {
                        end_opts = true;
                        continue;
                    }
                    if flag.starts_with('-') {
                        return;
                    }
                }
            }
        }
        end_opts = true;
        match parsed {
            Some(operand) => {
                if let Some(name) = indexed_name(&operand) {
                    unset_element(state, name, case_ctx);
                } else if identifier(&operand) {
                    unset_name(state, &operand, case_ctx);
                } else if word.dynamic || has_expansion(word.raw) {
                    if let Some(resolved) =
                        extract_single_var(word.raw).and_then(|var| case_ctx.resolve(var))
                    {
                        let single = resolved.names.len() == 1;
                        for name in resolved.names {
                            if resolved.case_insensitive {
                                state.touch_ci_empty(&name);
                            }
                            if single {
                                unset_name(state, &name, case_ctx);
                            } else {
                                unset_element(state, &name, case_ctx);
                            }
                        }
                    } else {
                        dynamic = true;
                    }
                }
            }
            None => dynamic = true,
        }
    }
    if dynamic {
        state.record_dynamic_unset();
    }
}

pub(super) fn single_word(raw: &str) -> Option<String> {
    let mut parts = split_quoted_words(raw, Whitespace::Ascii)?;
    (parts.len() == 1).then(|| parts.swap_remove(0))
}

fn unset_element(state: &mut State, name: &str, case_ctx: &mut dyn CaseUnsetCtx) {
    state.touch(name);
    if state.arrays.contains_key(name) {
        state.set_known(name, ArrayState::MaybeEmpty);
    }
    case_ctx.invalidate(name);
}

fn unset_name(state: &mut State, name: &str, case_ctx: &mut dyn CaseUnsetCtx) {
    if state.arrays.contains_key(name) {
        state.write(name, false);
    } else {
        state.touch_empty(name);
    }
    case_ctx.invalidate(name);
}

pub(super) fn has_expansion(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let (mut i, mut double_quoted) = (0, false);
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'\'' if !double_quoted => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
                i += 1;
            }
            b'"' => {
                double_quoted = !double_quoted;
                i += 1;
            }
            b'$' | b'`' => return true,
            _ => i += 1,
        }
    }
    false
}

pub(super) fn indexed_name(word: &str) -> Option<&str> {
    let (name, index) = word.strip_suffix(']')?.split_once('[')?;
    (identifier(name) && !index.is_empty() && !index.contains(['[', ']'])).then_some(name)
}
