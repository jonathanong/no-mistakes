use super::{
    super::lex::{CaseArm, Word},
    case_unset::{has_catchall, CaseLiterals, ResolvedUnset},
    guard::Leave,
    state::State,
};

pub(super) struct Case {
    before: State,
    completed: Option<State>,
    carried: Option<State>,
    carried_test_next: bool,
    literals: CaseLiterals,
    seen_pattern: bool,
    in_arm: bool,
    arm_dead: bool,
    arm_leave: Leave,
    all_exit: bool,
    exhaustive: bool,
    optional: bool,
}

impl Case {
    pub fn new(before: State, optional: bool, subject_vars: Vec<(String, bool)>) -> Self {
        Self {
            before,
            completed: None,
            carried: None,
            carried_test_next: false,
            literals: CaseLiterals::new(subject_vars),
            seen_pattern: false,
            in_arm: false,
            arm_dead: false,
            arm_leave: Leave::None,
            all_exit: true,
            exhaustive: false,
            optional,
        }
    }

    pub fn touch_pattern_write(&mut self, name: &str) {
        self.before.touch(name);
        self.literals.invalidate_var(name);
    }

    pub fn enter_pattern(&mut self, state: &mut State) -> bool {
        if !self.seen_pattern {
            self.before = state.clone();
        }
        let test_pattern = !self.exhaustive || self.carried_test_next;
        if test_pattern {
            let mut pat = match (self.exhaustive, self.carried_test_next, &self.carried) {
                (true, _, Some(carried)) => carried.clone(),
                (false, true, Some(carried)) => {
                    let mut merged = self.before.clone();
                    merged.merge(carried);
                    merged
                }
                _ => self.before.clone(),
            };
            if self.seen_pattern {
                pat.mark_guarded_maybe();
            }
            *state = pat;
        }
        test_pattern
    }

    pub fn pattern(&mut self, words: &[Word<'_>], state: &mut State) {
        let first_arm = !self.seen_pattern;
        self.seen_pattern = true;
        self.carried_test_next = false;
        let already_exhaustive = self.exhaustive;
        let catchall = has_catchall(words);
        if catchall {
            self.exhaustive = true;
        }
        self.literals.pattern(words, already_exhaustive);
        if already_exhaustive {
            if let Some(carried) = self.carried.take() {
                *state = carried;
                self.arm_dead = false;
            } else {
                *state = self.before.clone();
                self.arm_dead = true;
            }
        } else if let Some(mut carried) = self.carried.take() {
            carried.merge(&self.before);
            *state = carried;
            self.arm_dead = false;
        } else {
            *state = self.before.clone();
            self.arm_dead = false;
        }
        if !catchall || !first_arm {
            state.mark_guarded_maybe();
        }
        self.in_arm = true;
        self.arm_leave = Leave::None;
    }

    pub(super) fn arm_is_dead(&self) -> bool {
        self.arm_dead
    }

    pub fn command(&mut self, leave: Leave) {
        if self.in_arm {
            self.arm_leave = self.arm_leave.combine(leave);
        }
    }

    pub fn arm_end(&mut self, kind: CaseArm, state: &mut State) {
        if !self.in_arm {
            return;
        }
        self.literals.arm_end(kind, self.arm_leave.is_leaving());
        if !self.arm_dead {
            if !self.arm_leave.is_leaving() {
                match kind {
                    CaseArm::Break => {
                        self.all_exit = false;
                        State::merge_into(&mut self.completed, state);
                        self.carried = None;
                        self.carried_test_next = false;
                    }
                    CaseArm::Fallthrough => {
                        self.carried = Some(state.clone());
                        self.carried_test_next = false;
                    }
                    CaseArm::TestNext => {
                        self.all_exit = false;
                        State::merge_into(&mut self.completed, state);
                        State::merge_into(&mut self.carried, state);
                        self.carried_test_next = true;
                    }
                }
            } else {
                self.all_exit &= self.arm_leave == Leave::Exit;
                if kind == CaseArm::Break {
                    self.carried = None;
                    self.carried_test_next = false;
                }
            }
        }
        self.in_arm = false;
        self.arm_dead = false;
        self.arm_leave = Leave::None;
        *state = self.before.clone();
    }

    pub fn finish(mut self, state: &mut State, optional: bool) -> Leave {
        if self.in_arm && !self.arm_dead {
            if !self.arm_leave.is_leaving() {
                self.all_exit = false;
                State::merge_into(&mut self.completed, state);
            } else {
                self.all_exit &= self.arm_leave == Leave::Exit;
            }
        }
        if let Some(carried) = self.carried.take() {
            self.all_exit = false;
            State::merge_into(&mut self.completed, &carried);
        }
        let leaves_all = self.exhaustive && self.completed.is_none() && !self.optional && !optional;
        let leave = if leaves_all {
            if self.all_exit {
                Leave::Exit
            } else {
                Leave::Flow
            }
        } else {
            Leave::None
        };
        let mut next = if self.exhaustive {
            self.completed.take().unwrap_or_else(|| self.before.clone())
        } else {
            self.before.clone()
        };
        if let Some(completed) = self.completed {
            next.merge(&completed);
        }
        if self.optional || optional {
            next.merge(&self.before);
        }
        next.restrict_guarded_maybe(&self.before.guarded_maybe);
        *state = next;
        leave
    }

    pub fn resolve_unset(&self, var: &str) -> Option<ResolvedUnset> {
        self.literals.resolve(var)
    }

    pub fn invalidate_var(&mut self, var: &str) {
        self.literals.invalidate_var(var);
    }
}
