use super::super::lex::{CaseArm, Word};
use super::frames::Frame;
use crate::fx::FxHashSet;

mod patterns;

pub(super) use patterns::{
    extract_derivation, extract_single_var, has_catchall, subject_vars_from_header,
};

pub(super) struct ResolvedUnset {
    pub names: FxHashSet<String>,
    pub case_insensitive: bool,
}

pub(super) trait CaseUnsetCtx {
    fn resolve(&self, var: &str) -> Option<ResolvedUnset>;
    fn invalidate(&mut self, var: &str);
}

impl CaseUnsetCtx for Vec<Frame<'_>> {
    fn resolve(&self, var: &str) -> Option<ResolvedUnset> {
        for frame in self.iter().rev() {
            match frame {
                Frame::Group(group) if group.function.is_some() => return None,
                Frame::Case(case) => {
                    if let Some(resolved) = case.resolve_unset(var) {
                        return Some(resolved);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn invalidate(&mut self, var: &str) {
        for frame in self.iter_mut().rev() {
            match frame {
                Frame::Group(group) if group.function.is_some() => return,
                Frame::Case(case) => case.invalidate_var(var),
                _ => {}
            }
        }
    }
}

type CarriedCase = (Option<FxHashSet<String>>, Vec<(String, bool)>);

pub(super) struct CaseLiterals {
    initial_vars: Vec<(String, bool)>,
    arm_vars: Vec<(String, bool)>,
    arm_literals: Option<FxHashSet<String>>,
    carried: Option<CarriedCase>,
    in_arm: bool,
}

impl CaseLiterals {
    pub fn new(subject_vars: Vec<(String, bool)>) -> Self {
        Self {
            initial_vars: subject_vars,
            arm_vars: Vec::new(),
            arm_literals: None,
            carried: None,
            in_arm: false,
        }
    }

    pub fn pattern(&mut self, words: &[Word<'_>], already_exhaustive: bool) {
        let pat = patterns::literal_case_patterns(words);
        let carried = self.carried.take();
        if already_exhaustive {
            let (lits, vars) = carried.unwrap_or_default();
            self.arm_literals = lits;
            self.arm_vars = vars;
        } else if let Some((carried_lits, carried_vars)) = carried {
            self.arm_literals = match (carried_lits, pat) {
                (Some(mut a), Some(b)) => {
                    a.extend(b);
                    Some(a)
                }
                _ => None,
            };
            self.arm_vars = self
                .initial_vars
                .iter()
                .filter_map(|(name, exact)| {
                    carried_vars
                        .iter()
                        .find(|(n, _)| n == name)
                        .map(|(_, c_exact)| (name.clone(), *exact && *c_exact))
                })
                .collect();
        } else {
            self.arm_literals = pat;
            self.arm_vars = self.initial_vars.clone();
        }
        self.in_arm = true;
    }

    pub fn arm_end(&mut self, kind: CaseArm, leaving: bool) {
        if !self.in_arm {
            return;
        }
        if !leaving {
            match kind {
                CaseArm::Break => self.carried = None,
                CaseArm::Fallthrough => {
                    self.carried = Some((self.arm_literals.clone(), self.arm_vars.clone()));
                }
                CaseArm::TestNext => {
                    let prev = self.carried.take().map(|(lits, _)| lits);
                    let merged = match (prev, &self.arm_literals) {
                        (Some(Some(mut a)), Some(b)) => {
                            a.extend(b.iter().cloned());
                            Some(a)
                        }
                        (None, Some(b)) => Some(b.clone()),
                        _ => None,
                    };
                    self.carried = Some((merged, self.arm_vars.clone()));
                }
            }
        } else if kind == CaseArm::Break {
            self.carried = None;
        }
        self.in_arm = false;
        self.arm_literals = None;
        self.arm_vars.clear();
    }

    pub fn invalidate_var(&mut self, var: &str) {
        if self.in_arm {
            self.arm_vars.retain(|(name, _)| name != var);
        }
    }

    pub fn resolve(&self, var: &str) -> Option<ResolvedUnset> {
        if !self.in_arm {
            return None;
        }
        let names = self.arm_literals.as_ref()?;
        let &(_, exact) = self.arm_vars.iter().find(|(name, _)| name == var)?;
        Some(ResolvedUnset {
            names: names.clone(),
            case_insensitive: !exact,
        })
    }
}
