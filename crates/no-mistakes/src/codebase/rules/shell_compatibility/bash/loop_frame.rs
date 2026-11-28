use super::{guard::Leave, state::State};
use crate::fx::FxHashSet;

const MAX_PASSES: u8 = 4;

pub(super) struct LoopKind {
    pub always_enters: bool,
    pub for_assigns_var: bool,
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
            if self.kind.for_assigns_var {
                if let Some(var) = self.loop_var {
                    state.write(var, true);
                }
            }
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
            self.invalidate_infinite();
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

    pub fn next_pass(&mut self, state: &mut State) -> Option<(usize, usize, usize)> {
        let mut back = self.back_edge.clone();
        if !self.body_leave.is_leaving() {
            State::merge_into(&mut back, state);
        }
        let back = back?;
        let mut next_entry = self.entry.clone();
        if self.kind.restart_at_header {
            next_entry.merge(&self.before);
        }
        next_entry.merge(&back);
        if let Some(var) = self.loop_var {
            if self.kind.for_assigns_var {
                next_entry.write(var, true);
            } else {
                next_entry.invalidate_condition_var(var);
            }
        }
        if !self.entry.weakened_by(&next_entry) {
            return None;
        }
        if self.pass >= MAX_PASSES {
            next_entry.widen_unanalyzable(self.empties, self.touches);
            if !self.entry.weakened_by(&next_entry) {
                return None;
            }
        }
        self.entry = next_entry;
        *state = self.entry.clone();
        (self.back_edge, self.break_exit, self.continue_exit) = (None, None, None);
        (self.body_leave, self.in_body) = (Leave::None, false);
        self.pass = self.pass.saturating_add(1);
        let restart = if self.kind.restart_at_header {
            self.header_index
        } else {
            self.do_index.unwrap_or(self.header_index)
        };
        Some((restart, self.hits_start, self.unanalyzable_start))
    }

    pub fn finish(self, state: &mut State, optional: bool) -> Leave {
        let opt = self.kind.optional || optional;
        if self.kind.infinite && !opt {
            return if let Some(broken) = self.break_exit {
                *state = broken;
                Leave::None
            } else {
                *state = self.before;
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
            *state = self.before;
            self.body_leave
        }
    }
}
