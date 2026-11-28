use super::{condition::Condition, write_sets::WriteSets};
use crate::fx::{FxHashMap, FxHashSet};

mod guards;
mod merge;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrayState {
    Nonempty,
    MaybeEmpty,
    Empty,
}

impl ArrayState {
    pub fn from_nonempty(nonempty: bool) -> Self {
        if nonempty {
            Self::Nonempty
        } else {
            Self::Empty
        }
    }

    pub fn is_nonempty(self) -> bool {
        self == Self::Nonempty
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum UnanalyzableReason {
    LoopWidening,
    DynamicUnset,
}

#[derive(Clone, Default)]
pub(super) struct State {
    pub arrays: FxHashMap<String, ArrayState>,
    pub guarded_maybe: FxHashSet<String>,
    pub unanalyzable: FxHashMap<String, UnanalyzableReason>,
    pub conditional: FxHashMap<String, Vec<Condition>>,
    pub derived_from: FxHashMap<String, (String, bool)>,
    pub active: bool,
    pub nounset_effect: Option<bool>,
    writes: Option<FxHashSet<String>>,
    empties: Option<FxHashSet<String>>,
    ci_empties: Option<FxHashSet<String>>,
    locals: Option<FxHashSet<String>>,
    dynamic_unset: bool,
}

pub(super) struct CollectedWrites {
    pub touches: FxHashSet<String>,
    pub empties: FxHashSet<String>,
    pub ci_empties: FxHashSet<String>,
    pub locals: FxHashSet<String>,
    pub dynamic_unset: bool,
}

impl State {
    pub fn set_nounset(&mut self, active: bool) {
        self.active = active;
        self.nounset_effect = Some(active);
    }

    pub fn write(&mut self, name: &str, nonempty: bool) {
        self.write_array(name, nonempty, false, false);
    }

    pub fn declare_local(&mut self, name: &str, empty: bool) {
        self.touch(name);
        if let Some(locals) = &mut self.locals {
            locals.insert(name.to_owned());
        }
        if empty {
            self.conditional.remove(name);
            self.unanalyzable.remove(name);
            self.guarded_maybe.remove(name);
            self.arrays.insert(name.to_owned(), ArrayState::Empty);
        }
    }

    pub fn write_array(&mut self, name: &str, nonempty: bool, is_local: bool, append: bool) {
        self.touch(name);
        if is_local {
            if let Some(locals) = &mut self.locals {
                locals.insert(name.to_owned());
            }
        }
        if !nonempty && !is_local && !append {
            if let Some(empties) = &mut self.empties {
                empties.insert(name.to_owned());
            }
        }
        if !append || nonempty {
            self.conditional.remove(name);
        }
        self.unanalyzable.remove(name);
        self.guarded_maybe.remove(name);
        self.arrays
            .insert(name.to_owned(), ArrayState::from_nonempty(nonempty));
    }

    pub fn collect_writes() -> Self {
        Self {
            writes: Some(FxHashSet::default()),
            empties: Some(FxHashSet::default()),
            ci_empties: Some(FxHashSet::default()),
            locals: Some(FxHashSet::default()),
            ..Self::default()
        }
    }

    pub fn touch(&mut self, name: &str) {
        if let Some(writes) = &mut self.writes {
            writes.insert(name.to_owned());
        }
        self.derived_from.remove(name);
        self.derived_from.retain(|_, (src, _)| src != name);
        self.invalidate_condition_var(name);
    }

    pub fn touch_empty(&mut self, name: &str) {
        self.touch(name);
        if let Some(empties) = &mut self.empties {
            empties.insert(name.to_owned());
        }
    }

    pub fn set_known(&mut self, name: &str, state: ArrayState) {
        self.unanalyzable.remove(name);
        self.guarded_maybe.remove(name);
        self.arrays.insert(name.to_owned(), state);
    }

    pub fn into_writes(self) -> CollectedWrites {
        CollectedWrites {
            touches: self.writes.unwrap_or_default(),
            empties: self.empties.unwrap_or_default(),
            ci_empties: self.ci_empties.unwrap_or_default(),
            locals: self.locals.unwrap_or_default(),
            dynamic_unset: self.dynamic_unset,
        }
    }

    pub fn invalidate(&mut self, empties: &FxHashSet<String>, touches: &FxHashSet<String>) {
        for name in empties {
            self.arrays.insert(name.clone(), ArrayState::Empty);
            self.unanalyzable.remove(name);
            self.guarded_maybe.remove(name);
            self.conditional.remove(name);
        }
        for name in touches {
            self.derived_from.remove(name);
            self.derived_from.retain(|_, (src, _)| src != name);
            self.invalidate_condition_var(name);
        }
    }

    pub fn forget_nonempty_for_function(&mut self, index: usize, write_sets: &WriteSets) {
        self.guarded_maybe.clear();
        for (name, state) in &mut self.arrays {
            if *state == ArrayState::Nonempty && write_sets.emptied_after(name, index) {
                *state = ArrayState::MaybeEmpty;
            }
        }
        self.conditional.retain(|name, conditions| {
            if write_sets.emptied_after(name, index) {
                return false;
            }
            conditions.retain(|cond| !cond.vars().any(|var| write_sets.touched_after(var, index)));
            !conditions.is_empty()
        });
        if write_sets.dynamic_unset_after(index) {
            self.record_dynamic_unset();
        }
    }

    pub fn true_arrays(&self) -> FxHashSet<String> {
        self.arrays
            .iter()
            .filter(|&(_, s)| s.is_nonempty())
            .map(|(name, _)| name.clone())
            .collect()
    }
}
