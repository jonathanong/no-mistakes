//! Schema-qualified relations participate in temporary-view invalidation.
use super::{Dependency, State};
use crate::codebase::postgres::decoded_parts;
use std::collections::BTreeSet;

impl State {
    pub fn drop_schema(&mut self, name: &str) {
        let parts = decoded_parts(name);
        let [schema] = parts.as_slice() else {
            return;
        };
        let mut dropped: BTreeSet<Vec<String>> = self
            .physical_views
            .keys()
            .filter(|name| name.len() == 1 || name[name.len() - 2] == *schema)
            .cloned()
            .collect();
        dropped.extend(
            self.relations
                .values()
                .chain(self.physical_views.values())
                .flat_map(|dependencies| dependencies.iter())
                .filter_map(|dependency| match dependency {
                    Dependency::Physical(source)
                        if source.len() >= 2 && source[source.len() - 2] == *schema =>
                    {
                        Some(source.clone())
                    }
                    _ => None,
                }),
        );
        self.drop_physical(dropped, true);
    }
}
