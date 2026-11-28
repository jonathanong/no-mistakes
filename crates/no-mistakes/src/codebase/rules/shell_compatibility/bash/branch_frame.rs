use super::{
    super::lex::{Boundary, Word},
    condition::{BranchConditions, Condition},
    guard::{Guard, Leave},
    state::{ArrayState, State},
    write_sets::WriteSets,
};
use finish::contradicts;

mod finish;

pub(super) struct Branch<'a> {
    start_index: usize,
    before: State,
    completed: Option<State>,
    condition: Option<State>,
    guard: Option<Guard<'a>>,
    conds: BranchConditions,
    in_condition: bool,
    dead_arm: bool,
    otherwise: bool,
    leave: Leave,
    all_exit: bool,
    optional: bool,
}

impl<'a> Branch<'a> {
    pub fn new(
        start_index: usize,
        before: State,
        guard: Option<Guard<'a>>,
        optional: bool,
    ) -> Self {
        Self {
            start_index,
            before,
            completed: None,
            condition: None,
            guard,
            conds: BranchConditions::default(),
            in_condition: true,
            dead_arm: false,
            otherwise: false,
            leave: Leave::None,
            all_exit: true,
            optional,
        }
    }

    pub fn in_condition(&self) -> bool {
        self.in_condition
    }

    pub fn arm_is_dead(&self) -> bool {
        self.dead_arm
    }

    pub fn invalidate_condition(&mut self) {
        if self.in_condition {
            self.conds.builder.invalidate();
        }
    }

    pub fn condition_command(
        &mut self,
        words: &[&Word<'_>],
        outgoing: Boundary<'_>,
        redirected: bool,
    ) {
        if self.in_condition {
            self.conds.builder.push(words, outgoing, redirected);
        }
    }

    pub fn command(&mut self, leave: Leave) {
        if self.in_condition {
            self.guard = None;
        } else {
            self.leave = self.leave.combine(leave);
        }
    }

    pub fn then(&mut self, state: &mut State) {
        self.condition = Some(state.clone());
        self.dead_arm = false;
        if let Some(guard) = self.guard {
            if contradicts(guard, state, true) {
                self.dead_arm = true;
            } else {
                state.set_known(guard.name, ArrayState::from_nonempty(!guard.when_empty));
            }
        }
        let (current, conjuncts) = self.conds.builder.finish();
        for conjunct in &conjuncts {
            state.promote_condition(conjunct);
        }
        if !self.conds.prior_broken {
            if let Some(cond) = current
                .as_ref()
                .and_then(|c| Condition::with_priors(&self.conds.prior_negated, Some(c)))
            {
                state.promote_condition(&cond);
                self.conds.active = Some(cond);
            }
        }
        state.mark_guarded_maybe();
        self.conds.current_raw = current;
        self.in_condition = false;
    }

    pub fn next_branch(&mut self, state: &mut State, otherwise: bool) {
        self.save(state);
        if !self.leave.is_leaving() {
            if let Some(raw) = self.conds.current_raw.take() {
                self.conds.prior_negated.push(raw.negated());
            } else {
                self.conds.prior_broken = true;
            }
        } else {
            self.conds.current_raw = None;
        }
        let base = self.condition.as_ref().unwrap_or(&self.before);
        let else_dead = otherwise && self.guard.is_some_and(|g| contradicts(g, base, false));
        *state = if else_dead {
            base.clone()
        } else {
            self.fallthrough()
        };
        self.guard = None;
        self.leave = Leave::None;
        self.dead_arm = else_dead;
        self.in_condition = !otherwise;
        self.otherwise = otherwise;
        self.conds.builder.reset();
        if otherwise && !else_dead {
            if !self.conds.prior_broken && !self.conds.prior_negated.is_empty() {
                if let Some(cond) = Condition::with_priors(&self.conds.prior_negated, None) {
                    state.promote_condition(&cond);
                    self.conds.active = Some(cond);
                }
            }
            state.mark_guarded_maybe();
        }
    }
}
