use super::{
    super::{
        frames::{Frame, Group},
        state::ArrayState,
        write_sets::WriteSets,
    },
    Flow,
};
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Clone)]
pub(in super::super) struct FnSummaries<'a> {
    pub(super) exits: FxHashSet<&'a str>,
    pub(super) nonempty: FxHashMap<&'a str, FxHashSet<String>>,
    pub(super) nounset: FxHashMap<&'a str, bool>,
    pub(super) redirected: FxHashSet<&'a str>,
    pub(super) unsets: FxHashMap<&'a str, FxHashMap<String, bool>>,
    pub(super) dynamic_unsets: FxHashSet<&'a str>,
}

impl<'a> Flow<'a> {
    pub(super) fn snapshot_fns(&self) -> FnSummaries<'a> {
        FnSummaries {
            exits: self.fn_exits.clone(),
            nonempty: self.fn_nonempty.clone(),
            nounset: self.fn_nounset.clone(),
            redirected: self.fn_redirected.clone(),
            unsets: self.fn_unsets.clone(),
            dynamic_unsets: self.fn_dynamic_unsets.clone(),
        }
    }

    pub(super) fn restore_fns(&mut self, saved: FnSummaries<'a>) {
        self.fn_exits = saved.exits;
        self.fn_nonempty = saved.nonempty;
        self.fn_nounset = saved.nounset;
        self.fn_redirected = saved.redirected;
        self.fn_unsets = saved.unsets;
        self.fn_dynamic_unsets = saved.dynamic_unsets;
    }

    pub(super) fn in_conditional_frame(&self) -> bool {
        self.frames
            .iter()
            .rev()
            .take_while(|f| !matches!(f, Frame::Group(g) if g.isolated))
            .any(|f| !matches!(f, Frame::Group(g) if !g.optional))
    }

    pub(super) fn record_function_summary(
        &mut self,
        name: &'a str,
        group: &Group<'a>,
        def_redirected: bool,
        write_sets: &WriteSets,
    ) {
        let (exits, nonempty, nounset) = group.function_summary(
            &self.state,
            write_sets.touches(group.open_index),
            write_sets.locals(group.open_index),
        );
        if exits && !def_redirected {
            self.fn_exits.insert(name);
            self.fn_nonempty.remove(name);
            self.fn_nounset.remove(name);
        } else {
            self.fn_exits.remove(name);
            if nonempty.is_empty() {
                self.fn_nonempty.remove(name);
            } else {
                self.fn_nonempty.insert(name, nonempty);
            }
            if let Some(active) = nounset {
                self.fn_nounset.insert(name, active);
            } else {
                self.fn_nounset.remove(name);
            }
        }
        if def_redirected {
            self.fn_redirected.insert(name);
        } else {
            self.fn_redirected.remove(name);
        }
        if group.unset_fns.is_empty() {
            self.fn_unsets.remove(name);
        } else {
            self.fn_unsets.insert(name, group.unset_fns.clone());
        }
        if group.dynamic_fn_unset {
            self.fn_dynamic_unsets.insert(name);
        } else {
            self.fn_dynamic_unsets.remove(name);
        }
    }

    pub(super) fn apply_function_call(&mut self, called: &str, conditional: bool) {
        if let Some(nonempty) = self.fn_nonempty.get(called) {
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
        let Some(unsets) = self.fn_unsets.get(called).cloned() else {
            return;
        };
        let call_cond = conditional || self.in_conditional_frame();
        for (name, must_unset) in unsets {
            self.unset_function_name(&name, call_cond || !must_unset);
        }
    }
}
