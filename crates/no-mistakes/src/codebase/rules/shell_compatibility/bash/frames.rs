use super::{
    branch_frame::Branch,
    case_frame::Case,
    guard::{Guard, Leave},
    loop_frame::Repeat,
    state::{ArrayState, State},
};
use crate::fx::{FxHashMap, FxHashSet};

pub(super) enum Frame<'a> {
    If(Box<Branch<'a>>),
    Case(Box<Case>),
    Repeat(Box<Repeat<'a>>),
    Group(Box<Group<'a>>),
}

pub(super) struct Group<'a> {
    pub open_index: usize,
    pub before: State,
    pub isolated: bool,
    pub optional: bool,
    pub function: Option<&'a str>,
    pub enters_function: bool,
    pub guard: Option<Guard<'a>>,
    pub leave: Leave,
    pub return_true: Option<FxHashSet<String>>,
    pub return_nounset: Option<Option<bool>>,
    pub saved_fns: Option<super::flow::FnSummaries<'a>>,
    pub unset_fns: FxHashMap<String, bool>,
    pub dynamic_fn_unset: bool,
    pub pending_calls: Vec<(String, bool)>,
}

pub(super) fn merge_nounset_effect(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

impl Group<'_> {
    pub fn record_return(&mut self, state: &State) {
        let current = state.true_arrays();
        if let Some(existing) = &mut self.return_true {
            existing.retain(|name| current.contains(name));
        } else {
            self.return_true = Some(current);
        }
        self.return_nounset = Some(match self.return_nounset {
            Some(existing) => merge_nounset_effect(existing, state.nounset_effect),
            None => state.nounset_effect,
        });
    }

    pub fn function_summary(
        &self,
        body: &State,
        touches: &FxHashSet<String>,
        locals: &FxHashSet<String>,
    ) -> (bool, FxHashSet<String>, Option<bool>) {
        if self.leave == Leave::Exit && self.return_true.is_none() {
            return (true, FxHashSet::default(), None);
        }
        let mut guaranteed = self.return_true.clone();
        let mut effect = self.return_nounset;
        if !self.leave.is_leaving() {
            let current = body.true_arrays();
            if let Some(existing) = &mut guaranteed {
                existing.retain(|name| current.contains(name));
            } else {
                guaranteed = Some(current);
            }
            effect = Some(match effect {
                Some(existing) => merge_nounset_effect(existing, body.nounset_effect),
                None => body.nounset_effect,
            });
        }
        let mut nonempty = guaranteed.unwrap_or_default();
        nonempty.retain(|name| touches.contains(name) && !locals.contains(name));
        (false, nonempty, effect.flatten())
    }

    pub fn finish(mut self, state: &mut State, completes_list: bool, redirected: bool) -> Leave {
        if self.isolated {
            *state = self.before;
            return Leave::None;
        }
        if self.optional || !completes_list || redirected {
            self.before.merge(state);
            *state = self.before;
        }
        if self.leave.is_leaving() && completes_list && !redirected {
            if let Some(guard) = self.guard {
                state.set_known(guard.name, ArrayState::Nonempty);
            }
            return if self.optional {
                Leave::None
            } else {
                self.leave
            };
        }
        Leave::None
    }
}
