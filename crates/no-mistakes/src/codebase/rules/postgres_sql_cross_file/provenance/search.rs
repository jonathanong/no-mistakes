use super::*;
use crate::fx::{FxHashMap, FxHashSet};
type Key = (usize, String, u32);
#[derive(Default)]
pub(super) struct Search {
    active: FxHashSet<Key>,
    #[cfg(feature = "test-instrumentation")]
    work: usize,
    done: FxHashMap<(Key, Vec<String>), Option<String>>,
}
impl Bindings<'_> {
    pub(super) fn search_env(
        &self,
        file: &Path,
        value: &ValueExpression,
        members: &[String],
        state: &mut Search,
    ) -> Option<String> {
        match value {
            ValueExpression::Member(base, key) => {
                let mut path = vec![key.clone()];
                path.extend_from_slice(members);
                self.search_env(file, base, &path, state)
            }
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => {
                let Some(binding) = self.value(file, scope.unwrap_or(0), name, *offset) else {
                    return env_from_chain(self.facts, file, value, members);
                };
                #[cfg(feature = "test-instrumentation")]
                {
                    state.work += 1;
                }
                let key = (binding.scope, binding.name.clone(), binding.declared_at);
                let query = (key.clone(), members.to_vec());
                if let Some(done) = state.done.get(&query) {
                    return done.clone();
                }
                if !state.active.insert(key.clone()) {
                    return None;
                }
                let result = self.search_env(file, &binding.value, members, state);
                state.active.remove(&key);
                state.done.insert(query, result.clone());
                result
            }
            ValueExpression::Awaited(inner) => self.search_env(file, inner, members, state),
            ValueExpression::Derived(parts) => parts
                .iter()
                .find_map(|part| self.search_env(file, part, members, state)),
            ValueExpression::Nullish(left, right)
            | ValueExpression::Default(left, right)
            | ValueExpression::Either(left, right) => self
                .search_env(file, left, members, state)
                .or_else(|| self.search_env(file, right, members, state)),
            _ => None,
        }
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
mod tests;
