use super::{ArrayState, Branch, Guard, Leave, State, WriteSets};

impl Branch<'_> {
    pub fn finish(
        mut self,
        end_index: usize,
        state: &mut State,
        optional: bool,
        write_sets: &WriteSets,
    ) -> Leave {
        self.save(state);
        let opt = self.optional || optional;
        let out_leave = if self.otherwise && self.completed.is_none() && !opt {
            if self.all_exit {
                Leave::Exit
            } else {
                Leave::Flow
            }
        } else {
            Leave::None
        };
        let mut next = if self.otherwise {
            self.completed.take().unwrap_or_else(|| self.before.clone())
        } else {
            self.fallthrough()
        };
        if let Some(completed) = self.completed {
            next.merge(&completed);
        }
        if !opt {
            for (cond, names) in self.conds.gained {
                if !cond
                    .vars()
                    .any(|var| write_sets.touched_in_range(var, self.start_index, end_index))
                {
                    for name in names {
                        next.record_condition(&name, cond.clone());
                    }
                }
            }
        } else {
            next.merge(&self.before);
        }
        next.restrict_guarded_maybe(&self.before.guarded_maybe);
        *state = next;
        out_leave
    }

    pub(super) fn save(&mut self, state: &State) {
        let active = self.conds.active.take();
        if self.dead_arm {
            return;
        }
        if !self.leave.is_leaving() || self.in_condition {
            self.all_exit = false;
            if let Some(cond) = active {
                let names = state.nonempty_under(&cond);
                if !names.is_empty() {
                    self.conds.gained.push((cond, names));
                }
            }
            State::merge_into(&mut self.completed, state);
        } else {
            self.all_exit &= self.leave == Leave::Exit;
        }
    }

    pub(super) fn fallthrough(&self) -> State {
        let mut state = self
            .condition
            .clone()
            .unwrap_or_else(|| self.before.clone());
        if let Some(guard) = self.guard {
            state.set_known(guard.name, ArrayState::from_nonempty(guard.when_empty));
        }
        state
    }
}

pub(super) fn contradicts(guard: Guard<'_>, state: &State, taken: bool) -> bool {
    let expects_empty = guard.when_empty == taken;
    matches!(
        (expects_empty, state.arrays.get(guard.name).copied()),
        (true, Some(ArrayState::Nonempty)) | (false, Some(ArrayState::Empty))
    )
}
