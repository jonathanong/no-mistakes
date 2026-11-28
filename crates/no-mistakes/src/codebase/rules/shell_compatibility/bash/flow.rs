use super::super::lex::Boundary;
use super::{
    condition::Condition,
    frames::Frame,
    guard::{loop_levels, Guard, Leave},
    state::{ArrayState, State},
    write_sets::{Command, WriteSets},
};
use crate::fx::{FxHashMap, FxHashSet};

mod boundaries;
mod default;
mod finish_loop;
mod fn_summary;
mod observe;
mod prefix;
mod unset_fn;

pub(super) use fn_summary::FnSummaries;

pub(super) struct Flow<'a> {
    pub state: State,
    pub unanalyzable: Vec<(usize, String)>,
    frames: Vec<Frame<'a>>,
    incoming: Boundary<'a>,
    pending: Option<Guard<'a>>,
    pending_cond: Option<Condition>,
    seen_functions: FxHashMap<&'a str, usize>,
    redefined_functions: FxHashSet<&'a str>,
    fn_exits: FxHashSet<&'a str>,
    fn_nonempty: FxHashMap<&'a str, FxHashSet<String>>,
    fn_nounset: FxHashMap<&'a str, bool>,
    fn_redirected: FxHashSet<&'a str>,
    fn_unsets: FxHashMap<&'a str, FxHashMap<String, bool>>,
    fn_dynamic_unsets: FxHashSet<&'a str>,
    restart: Option<(usize, usize, usize)>,
}

impl<'a> Flow<'a> {
    pub fn command(
        &mut self,
        index: usize,
        command: &Command<'a>,
        write_sets: &'a WriteSets,
        hits_len: usize,
        mut analyze: impl FnMut(&mut State, &mut Vec<Frame<'a>>),
    ) -> Option<(usize, usize, usize)> {
        use Boundary::{And, CaseArm, End, Or};
        let words = &command.words;
        let scope = command.scope;
        let outgoing = command.boundary;
        let header_redirected = words.iter().any(|word| word.redirect);
        if self.handle_boundary(
            index,
            outgoing,
            words.is_empty(),
            header_redirected,
            write_sets,
            &mut analyze,
        ) {
            return None;
        }
        if outgoing == Boundary::CasePattern {
            let eval_pattern = match self.frames.last_mut() {
                Some(Frame::Case(case)) => case.enter_pattern(&mut self.state),
                _ => true,
            };
            self.reset_list();
            if eval_pattern {
                analyze(&mut self.state, &mut self.frames);
            }
            if let Some(Frame::Case(case)) = self.frames.last_mut() {
                case.pattern(words, &mut self.state);
            }
            return None;
        }
        let redirected = words.iter().any(|word| word.redirect);
        let words: Vec<_> = words.iter().filter(|word| !word.redirect).collect();
        let (words, compound) =
            self.prefix(index, &words, outgoing, redirected, write_sets, hits_len);
        if let Some(restart) = self.restart.take() {
            return Some(restart);
        }
        let incoming = self.incoming;
        if let Some(Frame::If(branch)) = self.frames.last_mut() {
            if branch.in_condition() {
                branch.condition_command(words, outgoing, redirected);
            }
        }
        let leave = if !redirected && matches!(outgoing, End | CaseArm(_)) {
            super::guard::leaves(
                words,
                scope,
                &self.frames,
                &self.fn_exits,
                &self.seen_functions,
            )
        } else {
            Leave::None
        };
        if !compound {
            let unconditional = if incoming == End { leave } else { Leave::None };
            self.observe_statement(unconditional);
        }
        if leave == Leave::Flow {
            let head = words.first().map(|w| w.raw);
            if head == Some("return") {
                self.record_function_return();
            } else {
                self.record_loop_jump(loop_levels(words), head == Some("break"));
            }
        }

        let conditional_unset = incoming != End
            || outgoing == Boundary::Other
            || redirected
            || self.in_conditional_frame();
        self.forget_function_unset(words, conditional_unset);
        self.run_command(index, outgoing, redirected, leave, write_sets, analyze);
        if leave.is_leaving() {
            if let Some(guard) = self.pending {
                self.state.set_known(guard.name, ArrayState::Nonempty);
            }
        }
        self.pending = (incoming == End && !compound && !redirected)
            .then(|| Guard::parse(words))
            .flatten()
            .filter(|guard| matches!(outgoing, And | Or) && guard.when_empty == (outgoing == And));
        self.pending_cond = (incoming == End && outgoing == And && !compound && !redirected)
            .then(|| Condition::from_words(words))
            .flatten();
        self.incoming = outgoing;
        None
    }

    fn run_command(
        &mut self,
        index: usize,
        outgoing: Boundary<'a>,
        redirected: bool,
        leave: Leave,
        write_sets: &WriteSets,
        mut analyze: impl FnMut(&mut State, &mut Vec<Frame<'a>>),
    ) {
        let incoming = self.incoming;
        let called = write_sets.called_function(index);
        let is_callback = write_sets.call_is_callback(index);
        let conditional = matches!(incoming, Boundary::And | Boundary::Or | Boundary::Other)
            || outgoing == Boundary::Other
            || redirected
            || is_callback
            || called.is_some_and(|name| self.fn_redirected.contains(name));
        let active_cond = (incoming == Boundary::And)
            .then(|| self.pending_cond.take())
            .flatten();
        let before = conditional.then(|| self.state.clone());
        if let Some(cond) = &active_cond {
            self.state.promote_condition(cond);
        }
        if matches!(incoming, Boundary::And | Boundary::Or) {
            if let Some(guard) = self.pending {
                self.state.set_known(guard.name, ArrayState::Empty);
            }
            self.state.mark_guarded_maybe();
        }
        analyze(&mut self.state, &mut self.frames);
        self.state.invalidate(
            write_sets.call_empties(index),
            write_sets.call_touches(index),
        );
        self.state.invalidate_ci(write_sets.call_ci_empties(index));
        if write_sets.call_dynamic_unset(index) {
            self.state.record_dynamic_unset();
        }
        if let Some(called) = called {
            self.apply_function_call(called, conditional);
        }
        if let Some(mut before) = before {
            let gained = active_cond.filter(|cond| {
                outgoing == Boundary::End
                    && !redirected
                    && !is_callback
                    && !leave.is_leaving()
                    && !cond
                        .vars()
                        .any(|var| write_sets.touched_in_range(var, index, index))
            });
            let gained_names = gained
                .as_ref()
                .map_or_else(Vec::new, |cond| self.state.nonempty_under(cond));
            before.merge(&self.state);
            if let Some(cond) = gained {
                for name in gained_names {
                    before.record_condition(&name, cond.clone());
                }
            }
            self.state = before;
        }
    }
}
