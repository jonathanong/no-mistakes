use super::{
    super::{
        super::lex::Boundary,
        frames::Frame,
        guard::Leave,
        state::{ArrayState, State},
        write_sets::WriteSets,
    },
    Flow,
};

impl<'a> Flow<'a> {
    pub(super) fn observe_statement(&mut self, leave: Leave) {
        match self.frames.last_mut() {
            Some(Frame::If(branch)) => branch.command(leave),
            Some(Frame::Case(case)) => case.command(leave),
            Some(Frame::Repeat(repeat)) => repeat.command(leave),
            Some(Frame::Group(group)) => group.leave = group.leave.combine(leave),
            None => {}
        }
    }

    pub(super) fn run_command(
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
            self.apply_function_call(called, conditional, write_sets);
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
