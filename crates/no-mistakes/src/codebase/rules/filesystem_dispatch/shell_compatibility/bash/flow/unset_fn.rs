use super::{
    super::{
        super::lex::{identifier, Word},
        assign::{has_expansion, single_word},
        frames::Frame,
    },
    Flow,
};

fn static_word(word: &Word<'_>) -> Option<String> {
    (!word.dynamic && !has_expansion(word.raw))
        .then(|| single_word(word.raw))
        .flatten()
}

impl<'a> Flow<'a> {
    pub(super) fn forget_function_unset(&mut self, words: &[&Word<'_>], conditional: bool) {
        let mut iter = words.iter().copied().filter(|word| !word.redirect);
        if iter.next().is_none_or(|word| word.raw != "unset") {
            return;
        }
        let Some(second) = iter.next() else {
            return;
        };
        let Some(second_val) = static_word(second) else {
            return;
        };
        if second_val == "-f" {
            for word in iter {
                if let Some(name) = static_word(word) {
                    if name != "--" && identifier(&name) {
                        self.unset_function_name(&name, conditional);
                    }
                } else {
                    self.unset_dynamic_functions();
                }
            }
            return;
        }
        if second_val.starts_with('-') && second_val != "--" {
            return;
        }
        let mut consumed = Vec::new();
        let mut end_opts = second_val == "--";
        if !end_opts {
            self.clear_plain_unset(&second_val, conditional, &mut consumed);
        }
        for word in iter {
            if let Some(name) = static_word(word) {
                if !end_opts && name == "--" {
                    end_opts = true;
                    continue;
                }
                end_opts = true;
                self.clear_plain_unset(&name, conditional, &mut consumed);
            }
        }
    }

    fn clear_plain_unset(&mut self, name: &str, conditional: bool, consumed: &mut Vec<String>) {
        if !identifier(name) {
            return;
        }
        if self.state.arrays.contains_key(name) && !consumed.iter().any(|item| item == name) {
            consumed.push(name.to_owned());
            return;
        }
        self.unset_function_name(name, conditional);
    }

    fn ensure_isolated_snapshot(&mut self) -> Option<&mut super::super::frames::Group<'a>> {
        let needs = self.frames.iter().rev().find_map(|frame| match frame {
            Frame::Group(group) if group.isolated => Some(group.saved_fns.is_none()),
            _ => None,
        });
        let snap = (needs == Some(true)).then(|| self.snapshot_fns());
        let group = self.frames.iter_mut().rev().find_map(|frame| match frame {
            Frame::Group(group) if group.isolated => Some(group.as_mut()),
            _ => None,
        })?;
        if let Some(snap) = snap {
            group.saved_fns = Some(snap);
        }
        Some(group)
    }

    pub(super) fn unset_function_name(&mut self, name: &str, conditional: bool) {
        if let Some(group) = self.ensure_isolated_snapshot() {
            if group.function.is_some() {
                let entry = group
                    .unset_fns
                    .entry(name.to_owned())
                    .or_insert(!conditional);
                *entry |= !conditional;
            }
        }
        let keep_enable = conditional && self.fn_nounset.get(name) == Some(&true);
        self.clear_function_inner(name, keep_enable);
    }

    pub(super) fn unset_dynamic_functions(&mut self) {
        if let Some(group) = self.ensure_isolated_snapshot() {
            if group.function.is_some() {
                group.dynamic_fn_unset = true;
            }
        }
        self.fn_exits.clear();
        self.fn_nonempty.clear();
        self.fn_nounset.retain(|_, &mut active| active);
    }

    pub(super) fn clear_function(&mut self, name: &str) {
        self.clear_function_inner(name, false);
    }

    fn clear_function_inner(&mut self, name: &str, keep_nounset: bool) {
        self.fn_exits.remove(name);
        self.fn_nonempty.remove(name);
        if !keep_nounset {
            self.fn_nounset.remove(name);
        }
        self.fn_redirected.remove(name);
        self.fn_unsets.remove(name);
        self.fn_dynamic_unsets.remove(name);
    }
}
