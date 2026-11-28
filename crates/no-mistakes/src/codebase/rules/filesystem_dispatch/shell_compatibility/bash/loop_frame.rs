use super::{super::lex::Word, guard::Leave, state::State};
use crate::fx::{FxHashMap, FxHashSet};

mod replay;
pub(super) use replay::LoopPass;

pub(super) struct LoopKind {
    pub always_enters: bool,
    pub infinite: bool,
    pub restart_at_header: bool,
    pub optional: bool,
}

pub(super) struct Repeat<'a> {
    pub header_index: usize,
    do_index: Option<usize>,
    pub hits_start: usize,
    pub unanalyzable_start: usize,
    before: State,
    pub entry: State,
    after_cond: Option<State>,
    back_edge: Option<State>,
    break_exit: Option<State>,
    continue_exit: Option<State>,
    body_leave: Leave,
    kind: LoopKind,
    pub in_body: bool,
    pub pass: u8,
    loop_var: Option<&'a str>,
    empties: &'a FxHashSet<String>,
    touches: &'a FxHashSet<String>,
}

impl<'a> Repeat<'a> {
    pub fn new(
        (header_index, hits_start, unanalyzable_start): (usize, usize, usize),
        before: State,
        kind: LoopKind,
        loop_var: Option<&'a str>,
        empties: &'a FxHashSet<String>,
        touches: &'a FxHashSet<String>,
    ) -> Self {
        Self {
            header_index,
            do_index: None,
            hits_start,
            unanalyzable_start,
            entry: before.clone(),
            after_cond: None,
            before,
            back_edge: None,
            break_exit: None,
            continue_exit: None,
            body_leave: Leave::None,
            kind,
            in_body: false,
            pass: 0,
            loop_var,
            empties,
            touches,
        }
    }

    pub fn enter_body(&mut self, index: usize, counts: (usize, usize), state: &mut State) {
        if self.pass == 0 {
            self.do_index = Some(index);
            self.entry = state.clone();
            if !self.kind.restart_at_header {
                (self.hits_start, self.unanalyzable_start) = counts;
            }
        } else if !self.kind.restart_at_header {
            *state = self.entry.clone();
        }
        self.after_cond = Some(state.clone());
        self.in_body = true;
    }

    pub fn command(&mut self, leave: Leave) {
        if self.in_body {
            self.body_leave = self.body_leave.combine(leave);
        } else {
            self.kind.infinite = false;
            self.kind.always_enters = false;
        }
    }

    pub fn invalidate_infinite(&mut self) {
        self.kind.infinite = false;
        self.kind.always_enters = false;
    }

    pub fn record_continue(&mut self, state: &State) {
        State::merge_into(&mut self.back_edge, state);
        State::merge_into(&mut self.continue_exit, state);
    }

    pub fn record_break(&mut self, state: &State) {
        State::merge_into(&mut self.break_exit, state);
    }

    pub fn finish(mut self, state: &mut State, optional: bool, exhausted: bool) -> Leave {
        let leave = self.finish_iteration(state, optional);
        if exhausted && !leave.is_leaving() {
            let exit = state.clone();
            state.merge(&self.entry);
            state.keep_exit_findings_under_exhaustion(&exit);
            state.widen_unanalyzable(self.empties, self.touches);
        }
        leave
    }

    fn finish_iteration(&mut self, state: &mut State, optional: bool) -> Leave {
        let opt = self.kind.optional || optional;
        if self.kind.infinite && !opt {
            return if let Some(broken) = self.break_exit.take() {
                *state = broken;
                Leave::None
            } else {
                *state = std::mem::take(&mut self.before);
                Leave::Exit
            };
        }
        let mut exit = None;
        if self.kind.restart_at_header {
            if opt {
                State::merge_into(&mut exit, &self.before);
            }
            if !self.kind.infinite {
                let cond_exit = self.after_cond.as_ref().unwrap_or(&self.before);
                State::merge_into(&mut exit, cond_exit);
            }
        } else {
            if !self.kind.always_enters || opt {
                State::merge_into(&mut exit, &self.before);
            }
            if !self.body_leave.is_leaving() {
                State::merge_into(&mut exit, state);
            }
            if let Some(cont) = &self.continue_exit {
                State::merge_into(&mut exit, cont);
            }
        }
        if let Some(broken) = &self.break_exit {
            State::merge_into(&mut exit, broken);
        }
        if let Some(mut next) = exit {
            next.restrict_guarded_maybe(&self.before.guarded_maybe);
            *state = next;
            Leave::None
        } else {
            *state = std::mem::take(&mut self.before);
            self.body_leave
        }
    }
}

pub(super) fn is_infinite_loop(
    keyword: &str,
    rest: &[&Word<'_>],
    seen_functions: &FxHashMap<&str, usize>,
) -> bool {
    match (keyword, rest) {
        ("while", [word]) => {
            matches!(word.raw, ":" | "true") && !seen_functions.contains_key(word.raw)
        }
        ("until", [word]) => word.raw == "false" && !seen_functions.contains_key("false"),
        _ => false,
    }
}

pub(super) fn for_has_literal_item(rest: &[&Word<'_>]) -> bool {
    rest.get(1).is_some_and(|word| word.raw == "in") && rest[2..].iter().any(|word| !word.dynamic)
}
