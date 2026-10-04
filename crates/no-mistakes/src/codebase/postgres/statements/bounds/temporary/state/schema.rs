//! Schema-qualified relations participate in temporary-view invalidation.
use super::{Dependency, State};
use crate::codebase::postgres::decoded_parts;

impl State {
    pub fn drop_schema(&mut self, name: &str) {
        let parts = decoded_parts(name);
        let [schema] = parts.as_slice() else {
            return;
        };
        let removed = self
            .relations
            .iter()
            .filter(|(_, dependencies)| {
                dependencies.iter().any(|dependency| {
                    matches!(dependency, Dependency::Physical(source)
                    if source.len() >= 2 && source[source.len() - 2] == *schema)
                })
            })
            .map(|(name, _)| name.clone())
            .collect();
        self.remove(removed, true);
    }
}
