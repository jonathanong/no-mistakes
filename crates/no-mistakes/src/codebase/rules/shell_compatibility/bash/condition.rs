use super::super::lex::{Boundary, Word};
use crate::fx::FxHashSet;

mod arith;
mod predicates;
pub(super) mod vars;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Condition {
    tokens: Vec<String>,
    vars: FxHashSet<String>,
}

impl Condition {
    pub fn from_words(words: &[&Word<'_>]) -> Option<Self> {
        if words.is_empty()
            || words
                .iter()
                .any(|word| word.raw.contains("$(") || word.raw.contains('`'))
            || predicates::is_uncorrelatable_predicate(words)
        {
            return None;
        }
        let tokens = words.iter().map(|word| word.raw.to_owned()).collect();
        let mut referenced = FxHashSet::default();
        vars::collect_vars(words, &mut referenced);
        if predicates::has_volatile_var(&referenced) {
            return None;
        }
        Some(Self {
            tokens,
            vars: referenced,
        })
    }

    pub fn chain(&mut self, separator: &str, other: Self) {
        self.tokens.push(separator.to_owned());
        self.tokens.extend(other.tokens);
        self.vars.extend(other.vars);
    }

    pub fn negated(&self) -> Self {
        let mut tokens = Vec::with_capacity(self.tokens.len() + 1);
        tokens.push("!not".to_owned());
        tokens.extend(self.tokens.iter().cloned());
        Self {
            tokens,
            vars: self.vars.clone(),
        }
    }

    pub fn with_priors(priors: &[Self], current: Option<&Self>) -> Option<Self> {
        if priors.is_empty() {
            return current.cloned();
        }
        let mut tokens = Vec::new();
        let mut referenced = FxHashSet::default();
        for prior in priors {
            tokens.push("!prior".to_owned());
            tokens.extend(prior.tokens.iter().cloned());
            referenced.extend(prior.vars.iter().cloned());
        }
        if let Some(current) = current {
            tokens.push("!arm".to_owned());
            tokens.extend(current.tokens.iter().cloned());
            referenced.extend(current.vars.iter().cloned());
        }
        Some(Self {
            tokens,
            vars: referenced,
        })
    }

    pub fn references(&self, name: &str) -> bool {
        self.vars.contains(name)
    }

    pub fn vars(&self) -> impl Iterator<Item = &str> {
        self.vars.iter().map(String::as_str)
    }
}

#[derive(Default)]
pub(super) struct ConditionBuilder {
    state: BuildState,
    sep: Option<&'static str>,
    conjuncts: Vec<Condition>,
    pure_and: bool,
}

#[derive(Default)]
enum BuildState {
    #[default]
    Empty,
    Ready(Condition),
    Invalid,
}

impl ConditionBuilder {
    pub fn reset(&mut self) {
        self.state = BuildState::Empty;
        self.sep = None;
        self.conjuncts.clear();
        self.pure_and = true;
    }

    pub fn invalidate(&mut self) {
        self.state = BuildState::Invalid;
        self.conjuncts.clear();
    }

    pub fn push(&mut self, words: &[&Word<'_>], outgoing: Boundary<'_>, redirected: bool) {
        if redirected {
            self.invalidate();
            return;
        }
        let Some(part) = Condition::from_words(words) else {
            self.invalidate();
            return;
        };
        match &mut self.state {
            BuildState::Empty => {
                self.conjuncts = vec![part.clone()];
                self.pure_and = true;
                self.state = BuildState::Ready(part);
            }
            BuildState::Ready(existing) => {
                if let Some(sep) = self.sep.take() {
                    self.pure_and &= sep == "&&";
                    if self.pure_and {
                        self.conjuncts.push(part.clone());
                    } else {
                        self.conjuncts.clear();
                    }
                    existing.chain(sep, part);
                } else {
                    self.invalidate();
                }
            }
            BuildState::Invalid => {}
        }
        self.sep = match outgoing {
            Boundary::And => Some("&&"),
            Boundary::Or => Some("||"),
            Boundary::End => None,
            _ => {
                self.invalidate();
                None
            }
        };
    }

    pub fn finish(&mut self) -> (Option<Condition>, Vec<Condition>) {
        let conjuncts = if self.pure_and {
            std::mem::take(&mut self.conjuncts)
        } else {
            self.conjuncts.clear();
            Vec::new()
        };
        let cond = match std::mem::replace(&mut self.state, BuildState::Invalid) {
            BuildState::Ready(cond) if self.sep.is_none() => Some(cond),
            _ => None,
        };
        (cond, conjuncts)
    }
}

#[derive(Default)]
pub(super) struct BranchConditions {
    pub builder: ConditionBuilder,
    pub current_raw: Option<Condition>,
    pub prior_negated: Vec<Condition>,
    pub prior_broken: bool,
    pub active: Option<Condition>,
    pub gained: Vec<(Condition, Vec<String>)>,
}
