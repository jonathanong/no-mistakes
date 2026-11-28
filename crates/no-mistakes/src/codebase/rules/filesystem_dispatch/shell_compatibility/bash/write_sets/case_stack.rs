use super::super::super::lex::{Boundary, CaseArm, Word};
use super::super::case_unset::{
    has_catchall, subject_vars_from_header, CaseLiterals, CaseUnsetCtx, ResolvedUnset,
};
use super::Command;
use crate::fx::FxHashMap;

enum CaseSlot {
    FnBarrier(FxHashMap<String, (String, bool)>),
    Case {
        literals: CaseLiterals,
        exhaustive: bool,
    },
}

#[derive(Default)]
pub(super) struct WriteCaseStack {
    slots: Vec<CaseSlot>,
    pub derived: FxHashMap<String, (String, bool)>,
}

impl WriteCaseStack {
    pub fn open_function(&mut self) {
        let saved = std::mem::take(&mut self.derived);
        self.slots.push(CaseSlot::FnBarrier(saved));
    }

    pub fn close_function(&mut self) {
        while let Some(slot) = self.slots.pop() {
            if let CaseSlot::FnBarrier(saved) = slot {
                self.derived = saved;
                return;
            }
        }
    }

    pub fn track_command(&mut self, words: &[Word<'_>]) {
        let non_redir: Vec<&Word<'_>> = words.iter().filter(|w| !w.redirect).collect();
        let mut slice = non_redir.as_slice();
        while let Some((first, rest)) = slice.split_first() {
            match first.raw {
                "!" | "then" | "else" | "do" => slice = rest,
                "case" => {
                    let vars = subject_vars_from_header(rest, &self.derived);
                    self.slots.push(CaseSlot::Case {
                        literals: CaseLiterals::new(vars),
                        exhaustive: false,
                    });
                    return;
                }
                "esac" => {
                    if matches!(self.slots.last(), Some(CaseSlot::Case { .. })) {
                        self.slots.pop();
                    }
                    return;
                }
                _ => return,
            }
        }
    }

    pub fn pattern(&mut self, words: &[Word<'_>]) {
        if let Some(CaseSlot::Case {
            literals,
            exhaustive,
        }) = self.slots.last_mut()
        {
            let already = *exhaustive;
            if has_catchall(words) {
                *exhaustive = true;
            }
            literals.pattern(words, already);
        }
    }

    pub fn arm_end(&mut self, kind: CaseArm) {
        if let Some(CaseSlot::Case { literals, .. }) = self.slots.last_mut() {
            literals.arm_end(kind, false);
        }
    }
}

impl CaseUnsetCtx for WriteCaseStack {
    fn resolve(&self, var: &str) -> Option<ResolvedUnset> {
        for slot in self.slots.iter().rev() {
            match slot {
                CaseSlot::FnBarrier(_) => return None,
                CaseSlot::Case { literals, .. } => {
                    if let Some(resolved) = literals.resolve(var) {
                        return Some(resolved);
                    }
                }
            }
        }
        None
    }

    fn invalidate(&mut self, var: &str) {
        for slot in self.slots.iter_mut().rev() {
            match slot {
                CaseSlot::FnBarrier(_) => return,
                CaseSlot::Case { literals, .. } => literals.invalidate_var(var),
            }
        }
    }
}

pub(super) fn is_function_header(commands: &[Command<'_>], index: usize) -> bool {
    if commands[index].words.is_empty() {
        return false;
    }
    commands[index + 1..]
        .iter()
        .find(|cmd| !cmd.words.is_empty() || !matches!(cmd.boundary, Boundary::End))
        .is_some_and(|cmd| {
            matches!(
                cmd.boundary,
                Boundary::Open {
                    enters_function: true,
                    ..
                }
            )
        })
}
