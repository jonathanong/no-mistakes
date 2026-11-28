use super::{
    super::{
        super::{
            lex::{assignment, Word},
            read_targets::targets,
        },
        frames::Frame,
        state::ArrayState,
        write_sets::WriteSets,
    },
    Flow,
};
use crate::fx::FxHashSet;

impl<'a> Flow<'a> {
    pub(super) fn record_pending_call(
        &mut self,
        raw_words: &[Word<'_>],
        words: &[&Word<'_>],
        conditional: bool,
        write_sets: &WriteSets,
    ) {
        let Some(Frame::Group(group)) = self
            .frames
            .iter_mut()
            .rev()
            .find(|frame| matches!(frame, Frame::Group(g) if g.isolated))
        else {
            return;
        };
        if group.function.is_none() {
            return;
        }
        let target = targets(raw_words)
            .and_then(|t| t.callback.flatten())
            .map(|cb| (cb, true))
            .or_else(|| {
                words
                    .iter()
                    .copied()
                    .find(|w| assignment(w.raw).is_none())
                    .map(|w| (w.raw.to_owned(), false))
            });
        let Some((callee, is_cb)) = target else {
            return;
        };
        if !write_sets.is_known_function(&callee) {
            return;
        }
        if self.active_functions.contains_key(callee.as_str())
            && !self.fn_pending_calls.contains_key(callee.as_str())
        {
            return;
        }
        let call_cond = conditional || is_cb;
        if let Some((_, existing_cond)) = group.pending_calls.iter_mut().find(|(n, _)| n == &callee)
        {
            *existing_cond &= call_cond;
        } else {
            group.pending_calls.push((callee, call_cond));
        }
    }

    pub(super) fn apply_function_call(
        &mut self,
        called: &str,
        conditional: bool,
        write_sets: &WriteSets,
    ) {
        let mut visited = FxHashSet::default();
        self.apply_function_call_inner(called, conditional, write_sets, &mut visited);
    }

    fn apply_function_call_inner(
        &mut self,
        called: &str,
        conditional: bool,
        write_sets: &WriteSets,
        visited: &mut FxHashSet<String>,
    ) {
        if !visited.insert(called.to_owned()) {
            return;
        }
        let own_nonempty = self.fn_nonempty.get(called).cloned();
        let own_enabled = self.fn_nounset.get(called) == Some(&true);
        if let Some(nonempty) = &own_nonempty {
            for name in nonempty {
                self.state.set_known(name, ArrayState::Nonempty);
            }
        }
        if let Some(&active) = self.fn_nounset.get(called) {
            self.state.set_nounset(active);
        }
        if self.fn_dynamic_unsets.contains(called) {
            self.unset_dynamic_functions();
        }
        if let Some(unsets) = self.fn_unsets.get(called).cloned() {
            let call_cond = conditional || self.in_conditional_frame();
            for (name, must_unset) in unsets {
                self.unset_function_name(&name, call_cond || !must_unset);
            }
        }
        let Some(pending) = self.fn_pending_calls.get(called).cloned() else {
            return;
        };
        for (callee, call_cond) in pending {
            let Some(&callee_idx) = self.active_functions.get(callee.as_str()) else {
                continue;
            };
            let eff_cond =
                conditional || call_cond || self.fn_redirected.contains(callee.as_str());
            if eff_cond {
                let saved = self.state.clone();
                self.apply_callee_writes(callee_idx, write_sets, own_nonempty.as_ref());
                self.apply_function_call_inner(&callee, true, write_sets, visited);
                let mut merged = saved;
                merged.merge(&self.state);
                self.state = merged;
            } else {
                self.apply_callee_writes(callee_idx, write_sets, own_nonempty.as_ref());
                self.apply_function_call_inner(&callee, false, write_sets, visited);
            }
        }
        if own_enabled {
            self.state.set_nounset(true);
        }
    }

    fn apply_callee_writes(
        &mut self,
        callee_idx: usize,
        write_sets: &WriteSets,
        own_nonempty: Option<&FxHashSet<String>>,
    ) {
        let empties = write_sets.empties(callee_idx);
        self.state
            .invalidate(empties, write_sets.touches(callee_idx));
        self.state.invalidate_ci(write_sets.ci_empties(callee_idx));
        if write_sets.has_dynamic_unset(callee_idx) {
            self.state.record_dynamic_unset();
        }
        if let Some(nonempty) = own_nonempty {
            for name in empties {
                if nonempty.contains(name) {
                    self.state.set_known(name, ArrayState::MaybeEmpty);
                }
            }
        }
    }
}
