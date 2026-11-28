use super::super::condition::Condition;
use super::{ArrayState, State};
use crate::fx::FxHashSet;

impl State {
    pub fn clear_current_locals(&mut self) {
        self.must_locals.clear();
        self.may_locals.clear();
    }

    pub fn redeclare_local(&mut self, name: &str, array: bool) {
        if self.must_locals.contains(name) {
            if let Some(locals) = &mut self.locals {
                locals.insert(name.to_owned());
            }
            if array && !self.arrays.contains_key(name) {
                self.write_array(name, false, true, true);
            }
            return;
        }
        let maybe_preserved = self.may_locals.contains(name)
            && self
                .arrays
                .get(name)
                .is_some_and(|s| *s != ArrayState::Empty);
        let empty = array || self.arrays.contains_key(name);
        self.declare_local(name, empty);
        if maybe_preserved {
            self.set_known(name, ArrayState::MaybeEmpty);
        }
    }

    pub fn mark_guarded_maybe(&mut self) {
        for (name, &state) in &self.arrays {
            if state == ArrayState::MaybeEmpty {
                self.guarded_maybe.insert(name.clone());
            }
        }
    }

    pub fn restrict_guarded_maybe(&mut self, allowed: &FxHashSet<String>) {
        self.guarded_maybe.retain(|name| allowed.contains(name));
    }

    pub fn record_derivation(&mut self, lhs: &str, rhs_var: &str, exact: bool) {
        self.derived_from
            .insert(lhs.to_owned(), (rhs_var.to_owned(), exact));
    }

    pub fn touch_ci_empty(&mut self, name: &str) {
        let lower = name.to_ascii_lowercase();
        if let Some(ci) = &mut self.ci_empties {
            ci.insert(lower.clone());
        }
        let matched: Vec<String> = self
            .arrays
            .keys()
            .filter(|k| k.to_ascii_lowercase() == lower)
            .cloned()
            .collect();
        for key in matched {
            self.write(&key, false);
        }
        self.derived_from.retain(|k, (src, _)| {
            !k.eq_ignore_ascii_case(&lower) && !src.eq_ignore_ascii_case(&lower)
        });
        self.conditional.retain(|_, conditions| {
            conditions.retain(|cond| !cond.vars().any(|v| v.eq_ignore_ascii_case(&lower)));
            !conditions.is_empty()
        });
    }

    pub fn invalidate_ci(&mut self, ci_empties: &FxHashSet<String>) {
        if ci_empties.is_empty() {
            return;
        }
        let matched: Vec<String> = self
            .arrays
            .keys()
            .filter(|k| ci_empties.contains(&k.to_ascii_lowercase()))
            .cloned()
            .collect();
        for key in matched {
            self.arrays.insert(key.clone(), ArrayState::Empty);
            self.unanalyzable.remove(&key);
            self.guarded_maybe.remove(&key);
            self.conditional.remove(&key);
        }
        self.derived_from.retain(|k, (src, _)| {
            !ci_empties.contains(&k.to_ascii_lowercase())
                && !ci_empties.contains(&src.to_ascii_lowercase())
        });
        self.conditional.retain(|_, conditions| {
            conditions.retain(|cond| {
                !cond
                    .vars()
                    .any(|v| ci_empties.contains(&v.to_ascii_lowercase()))
            });
            !conditions.is_empty()
        });
    }

    pub fn record_condition(&mut self, name: &str, cond: Condition) {
        if !self.arrays.get(name).is_some_and(|s| s.is_nonempty()) {
            let entry = self.conditional.entry(name.to_owned()).or_default();
            if !entry.contains(&cond) {
                entry.push(cond);
            }
        }
    }

    pub fn promote_condition(&mut self, cond: &Condition) {
        let matched: Vec<String> = self
            .conditional
            .iter()
            .filter(|(_, conditions)| conditions.contains(cond))
            .map(|(name, _)| name.clone())
            .collect();
        for name in matched {
            self.unanalyzable.remove(&name);
            self.guarded_maybe.remove(&name);
            self.arrays.insert(name, ArrayState::Nonempty);
        }
    }

    pub fn nonempty_under(&self, cond: &Condition) -> Vec<String> {
        let mut names: Vec<String> = self.true_arrays().into_iter().collect();
        for (name, conditions) in &self.conditional {
            if conditions.contains(cond) && !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
    }

    pub(in super::super) fn invalidate_condition_var(&mut self, name: &str) {
        self.conditional.retain(|_, conditions| {
            conditions.retain(|cond| !cond.references(name));
            !conditions.is_empty()
        });
    }
}
