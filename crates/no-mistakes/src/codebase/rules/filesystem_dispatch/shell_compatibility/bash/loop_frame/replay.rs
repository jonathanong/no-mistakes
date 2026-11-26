use super::{Leave, Repeat, State};

const MAX_PASSES: u8 = 4;

pub(in super::super) enum LoopPass {
    Stable,
    Exhausted,
    Replay((usize, usize, usize)),
}

impl Repeat<'_> {
    pub(in super::super) fn next_pass(
        &mut self,
        state: &mut State,
        replay_allowed: bool,
    ) -> LoopPass {
        let Some(mut next_entry) = self.weaker_entry(state) else {
            return LoopPass::Stable;
        };
        if self.pass >= MAX_PASSES {
            next_entry.widen_unanalyzable(self.empties, self.touches);
        }
        if !replay_allowed {
            self.entry = next_entry;
            return LoopPass::Exhausted;
        }
        self.entry = next_entry;
        *state = self.entry.clone();
        self.back_edge = None;
        self.break_exit = None;
        self.continue_exit = None;
        self.body_leave = Leave::None;
        self.in_body = false;
        self.pass = self.pass.saturating_add(1);
        let restart = if self.kind.restart_at_header {
            self.header_index
        } else {
            self.do_index.unwrap_or(self.header_index)
        };
        LoopPass::Replay((restart, self.hits_start, self.unanalyzable_start))
    }

    fn weaker_entry(&self, state: &State) -> Option<State> {
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
            next_entry.invalidate_condition_var(var);
        }
        self.entry.weakened_by(&next_entry).then_some(next_entry)
    }
}
