use super::super::*;

/// Mutations usually affect one lexical scope, even through several aliases.
/// Keep its canonical name set inline; move it into the map for other scopes.
#[derive(Default)]
pub(in super::super) enum Affected {
    #[default]
    Empty,
    One {
        scope: usize,
        names: FxHashSet<String>,
    },
    Many(FxHashMap<usize, FxHashSet<String>>),
}
impl Affected {
    pub(in super::super) fn insert(&mut self, scope: usize, name: String) {
        match self {
            Self::Empty => {
                let mut names = fx_set();
                names.insert(name);
                *self = Self::One { scope, names };
            }
            Self::One {
                scope: first,
                names,
            } if *first == scope => {
                names.insert(name);
            }
            Self::One {
                scope: first,
                names,
            } => {
                let mut scopes: FxHashMap<usize, FxHashSet<String>> = fx_map();
                scopes.insert(*first, std::mem::take(names));
                scopes.entry(scope).or_default().insert(name);
                *self = Self::Many(scopes);
            }
            Self::Many(scopes) => {
                scopes.entry(scope).or_default().insert(name);
            }
        }
    }
    pub(in super::super) fn get(&self, scope: &usize) -> Option<&FxHashSet<String>> {
        match self {
            Self::One {
                scope: first,
                names,
            } => (*first == *scope).then_some(names),
            Self::Many(scopes) => scopes.get(scope),
            Self::Empty => None,
        }
    }
    pub(in super::super) fn iter(&self) -> impl Iterator<Item = (&usize, &FxHashSet<String>)> {
        let first = match self {
            Self::One { scope, names } => Some((scope, names)),
            _ => None,
        };
        let many = match self {
            Self::Many(scopes) => Some(scopes),
            _ => None,
        };
        first
            .into_iter()
            .chain(many.into_iter().flat_map(|scopes| scopes.iter()))
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn record_counts(&self, facts: &mut ValueFlowFacts) {
        if let Self::Many(scopes) = self {
            facts.reference_affected_scope_buffers += usize::from(scopes.capacity() > 0);
        }
    }
}
#[cfg(test)]
#[path = "affected_receiver_tests.rs"]
mod tests;
