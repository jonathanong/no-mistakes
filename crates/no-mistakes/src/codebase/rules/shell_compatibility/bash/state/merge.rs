use super::super::frames::merge_nounset_effect;
use super::{ArrayState, State, UnanalyzableReason};
use crate::fx::FxHashSet;

impl State {
    pub fn merge_into(slot: &mut Option<Self>, state: &Self) {
        if let Some(existing) = slot {
            existing.merge(state);
        } else {
            *slot = Some(state.clone());
        }
    }

    pub fn record_dynamic_unset(&mut self) {
        self.dynamic_unset = true;
        let nonempty: Vec<String> = self
            .arrays
            .iter()
            .filter(|&(_, s)| s.is_nonempty())
            .map(|(name, _)| name.clone())
            .collect();
        for name in nonempty {
            self.arrays.remove(&name);
            self.unanalyzable
                .insert(name, UnanalyzableReason::DynamicUnset);
        }
        self.conditional.clear();
    }

    pub fn record_mapfile_callback(&mut self) {
        self.dynamic_unset = true;
        let nonempty: Vec<String> = self
            .arrays
            .iter()
            .filter(|&(_, s)| s.is_nonempty())
            .map(|(name, _)| name.clone())
            .collect();
        for name in nonempty {
            self.arrays.remove(&name);
            self.unanalyzable
                .insert(name, UnanalyzableReason::MapfileCallback);
        }
        for state in self.arrays.values_mut() {
            if *state == ArrayState::Empty {
                *state = ArrayState::MaybeEmpty;
            }
        }
        self.conditional.clear();
        self.derived_from.clear();
        self.guarded_maybe.clear();
    }

    pub fn widen_unanalyzable(&mut self, empties: &FxHashSet<String>, touches: &FxHashSet<String>) {
        for name in empties {
            if self.arrays.get(name) == Some(&ArrayState::Nonempty) {
                self.arrays.remove(name);
                self.unanalyzable
                    .insert(name.clone(), UnanalyzableReason::LoopWidening);
            }
            self.conditional.remove(name);
        }
        for name in touches {
            self.invalidate_condition_var(name);
        }
    }

    pub fn weakened_by(&self, next: &Self) -> bool {
        let array = next.arrays.iter().any(|(name, &ns)| {
            self.arrays
                .get(name)
                .map_or(!ns.is_nonempty(), |&ss| ns != ss)
        }) || self.arrays.iter().any(|(name, &ss)| {
            ss.is_nonempty() && next.arrays.get(name) != Some(&ArrayState::Nonempty)
        });
        let guard_lost = self.guarded_maybe.iter().any(|name| {
            next.arrays.get(name) == Some(&ArrayState::MaybeEmpty)
                && !next.guarded_maybe.contains(name)
        });
        let deriv_lost = self
            .derived_from
            .iter()
            .any(|(k, v)| next.derived_from.get(k) != Some(v));
        let local_changed = self
            .must_locals
            .iter()
            .any(|name| !next.must_locals.contains(name))
            || next
                .may_locals
                .iter()
                .any(|name| !self.may_locals.contains(name));
        (!self.active && next.active)
            || array
            || guard_lost
            || deriv_lost
            || local_changed
            || self.conditional.iter().any(|(name, conds)| {
                next.arrays.get(name) != Some(&ArrayState::Nonempty)
                    && conds
                        .iter()
                        .any(|c| !next.conditional.get(name).is_some_and(|nc| nc.contains(c)))
            })
    }

    pub fn merge(&mut self, other: &Self) {
        self.active |= other.active;
        self.nounset_effect = merge_nounset_effect(self.nounset_effect, other.nounset_effect);
        self.must_locals
            .retain(|name| other.must_locals.contains(name));
        self.may_locals.extend(other.may_locals.iter().cloned());
        self.merge_conditional(other);
        self.derived_from
            .retain(|k, v| other.derived_from.get(k) == Some(v));
        let prev_arrays = self.arrays.clone();
        let prev_guarded = std::mem::take(&mut self.guarded_maybe);
        self.arrays.retain(
            |name, state| match (*state, other.arrays.get(name).copied()) {
                (ArrayState::Nonempty, Some(ArrayState::Nonempty)) => true,
                (ArrayState::Nonempty, Some(ArrayState::Empty | ArrayState::MaybeEmpty)) => {
                    *state = ArrayState::MaybeEmpty;
                    true
                }
                (ArrayState::Nonempty, None) => false,
                (ArrayState::Empty, Some(ArrayState::Nonempty | ArrayState::MaybeEmpty)) => {
                    *state = ArrayState::MaybeEmpty;
                    true
                }
                _ => true,
            },
        );
        for (name, &state) in &other.arrays {
            if !state.is_nonempty() && !self.arrays.contains_key(name) {
                self.arrays.insert(name.clone(), state);
            }
        }
        for (name, &state) in &self.arrays {
            if state == ArrayState::MaybeEmpty {
                let left_ok = prev_arrays.get(name) == Some(&ArrayState::Nonempty)
                    || prev_guarded.contains(name);
                let right_ok = other.arrays.get(name) == Some(&ArrayState::Nonempty)
                    || other.guarded_maybe.contains(name);
                if left_ok && right_ok {
                    self.guarded_maybe.insert(name.clone());
                }
            }
        }
        for (name, &reason) in &other.unanalyzable {
            if !self.arrays.get(name).is_some_and(|s| !s.is_nonempty()) {
                self.arrays.remove(name);
                self.unanalyzable.entry(name.clone()).or_insert(reason);
            }
        }
        self.unanalyzable
            .retain(|name, _| !self.arrays.get(name).is_some_and(|s| !s.is_nonempty()));
        for name in self.unanalyzable.keys() {
            self.arrays.remove(name);
            self.guarded_maybe.remove(name);
        }
    }

    fn merge_conditional(&mut self, other: &Self) {
        self.conditional.retain(|name, conds| {
            other.arrays.get(name) == Some(&ArrayState::Nonempty)
                || other.conditional.get(name).is_some_and(|other_conds| {
                    conds.retain(|cond| other_conds.contains(cond));
                    !conds.is_empty()
                })
        });
        for (name, other_conds) in &other.conditional {
            if self.arrays.get(name) == Some(&ArrayState::Nonempty)
                && other.arrays.get(name) != Some(&ArrayState::Nonempty)
            {
                self.conditional.insert(name.clone(), other_conds.clone());
            }
        }
    }
}
