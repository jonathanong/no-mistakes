use super::{
    super::lex::{assignment, command_words, Boundary, Word},
    case_unset::CaseUnsetCtx,
    remember_command,
    state::State,
};
use crate::fx::{FxHashMap, FxHashSet};
use case_stack::{is_function_header, WriteCaseStack};
use scope::{extend_scope_writes, finish_scope, record_global_writes, track_loop, Scope, Writes};

mod case_stack;
mod query;
mod scope;

pub(super) struct Command<'a> {
    pub words: Vec<Word<'a>>,
    pub scope: usize,
    pub boundary: Boundary<'a>,
}

#[derive(Default)]
pub(super) struct WriteSets {
    touches_by_scope: FxHashMap<usize, FxHashSet<String>>,
    empties_by_scope: FxHashMap<usize, FxHashSet<String>>,
    ci_empties_by_scope: FxHashMap<usize, FxHashSet<String>>,
    locals_by_scope: FxHashMap<usize, FxHashSet<String>>,
    dynamic_unset_scopes: FxHashSet<usize>,
    calls: FxHashMap<usize, (usize, String)>,
    last_call_by_fn: FxHashMap<usize, usize>,
    empty_positions: FxHashMap<String, Vec<usize>>,
    ci_empty_positions: FxHashMap<String, Vec<usize>>,
    dynamic_unset_positions: Vec<usize>,
    dynamic_touches: FxHashSet<String>,
    touches: FxHashMap<String, Vec<usize>>,
    range_touches: FxHashMap<String, Vec<usize>>,
    empty: FxHashSet<String>,
}

impl WriteSets {
    pub fn new(commands: &[Command<'_>]) -> Self {
        let mut summary = Self::default();
        let mut stack = Vec::new();
        let mut functions = FxHashMap::default();
        let mut cases = WriteCaseStack::default();
        for (index, command) in commands.iter().enumerate() {
            if summary.handle_scope_boundary(index, command, &mut stack, &mut functions, &mut cases)
            {
                continue;
            }
            cases.track_command(&command.words);
            let loop_var = track_loop(index, &command.words, &mut stack, &mut summary);
            let dynamic =
                command.scope > 0 || stack.iter().any(|w| matches!(w.scope, Scope::Repeat));
            let pos = if dynamic { usize::MAX } else { index };
            let called = (!is_function_header(commands, index))
                .then(|| {
                    command_words(&command.words)
                        .find(|word| assignment(word.raw).is_none())
                        .and_then(|word| functions.get(word.raw).map(|&fn_idx| (fn_idx, word.raw)))
                })
                .flatten();
            if let Some((fn_idx, name)) = called {
                summary.calls.insert(index, (fn_idx, name.to_owned()));
                summary
                    .last_call_by_fn
                    .entry(fn_idx)
                    .and_modify(|existing| *existing = (*existing).max(pos))
                    .or_insert(pos);
            }
            let mut state = State::collect_writes();
            state.derived_from = std::mem::take(&mut cases.derived);
            remember_command(&command.words, command.scope > 0, &mut state, &mut cases);
            if let Some(var) = loop_var {
                state.touch(var);
                cases.invalidate(var);
            }
            cases.derived = std::mem::take(&mut state.derived_from);
            let mut cmd = state.into_writes();
            cmd.touches
                .extend(summary.call_touches(index).iter().cloned());
            cmd.empties
                .extend(summary.call_empties(index).iter().cloned());
            cmd.ci_empties
                .extend(summary.call_ci_empties(index).iter().cloned());
            cmd.dynamic_unset |= summary.call_dynamic_unset(index);
            if let Some(scope) = stack.last_mut() {
                scope.locals.extend(cmd.locals.iter().cloned());
            }
            record_global_writes(&mut summary, &stack, &cmd, (index, pos, dynamic));
            extend_scope_writes(&mut stack, cmd);
        }
        summary
    }

    fn handle_scope_boundary<'a>(
        &mut self,
        index: usize,
        command: &Command<'a>,
        stack: &mut Vec<Writes<'a>>,
        functions: &mut FxHashMap<&'a str, usize>,
        cases: &mut WriteCaseStack,
    ) -> bool {
        match command.boundary {
            Boundary::CasePattern => {
                cases.pattern(&command.words);
                true
            }
            Boundary::CaseArm(kind) if command.words.is_empty() => {
                cases.arm_end(kind);
                true
            }
            Boundary::Unanalyzable(_) => true,
            Boundary::Open {
                isolated,
                function,
                enters_function,
                ..
            } => {
                if enters_function {
                    cases.open_function();
                }
                stack.push(Writes::new(index, Scope::Group { isolated, function }));
                true
            }
            Boundary::Close { .. } => {
                if let Some(writes) = stack.pop() {
                    if let Scope::Group {
                        function: Some((name, brace_body)),
                        ..
                    } = writes.scope
                    {
                        cases.close_function();
                        if brace_body {
                            functions.insert(name, writes.index);
                        } else {
                            functions.remove(name);
                        }
                    }
                    finish_scope(self, writes, stack.last_mut());
                }
                true
            }
            _ => false,
        }
    }
}
