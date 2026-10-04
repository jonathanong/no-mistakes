//! Permanent view dependency nodes share the request-local lifecycle state.
use super::{names::names_match, Dependency, State};
use std::collections::BTreeSet;

impl State {
    pub fn drop_physical(&mut self, mut dropped: BTreeSet<Vec<String>>, cascade: bool) {
        if !cascade
            && self
                .relations
                .values()
                .chain(self.physical_views.values())
                .flat_map(|dependencies| dependencies.iter())
                .any(|dependency| {
                    matches!(dependency, Dependency::Physical(source)
                        if dropped.iter().any(|name| names_match(source, name)))
                })
        {
            // PostgreSQL rejects RESTRICT while a known view still depends on the target.
            return;
        }
        if cascade {
            loop {
                let before = dropped.len();
                for (view, dependencies) in &self.physical_views {
                    if dependencies.iter().any(|dependency| {
                        matches!(dependency, Dependency::Physical(source)
                            if dropped.iter().any(|name| names_match(source, name)))
                    }) {
                        dropped.insert(view.clone());
                    }
                }
                if dropped.len() == before {
                    break;
                }
            }
            let temporary = self
                .relations
                .iter()
                .filter(|(_, dependencies)| {
                    dependencies.iter().any(|dependency| {
                        matches!(dependency, Dependency::Physical(source)
                            if dropped.iter().any(|name| names_match(source, name)))
                    })
                })
                .map(|(name, _)| name.clone())
                .collect();
            self.remove(temporary, true);
        }
        self.physical_views
            .retain(|view, _| !dropped.iter().any(|name| names_match(view, name)));
    }

    pub fn rename_physical(&mut self, old: &[String], new: &str) {
        for dependencies in self.physical_views.values_mut() {
            *dependencies = rename_dependencies(dependencies, old, new);
        }
    }
}

pub(super) fn rename_dependencies(
    dependencies: &BTreeSet<Dependency>,
    old: &[String],
    new: &str,
) -> BTreeSet<Dependency> {
    dependencies
        .iter()
        .flat_map(|dependency| match dependency {
            Dependency::Physical(parts) if names_match(parts, old) => {
                let mut renamed = parts.clone();
                renamed.pop();
                renamed.push(new.to_owned());
                let mut candidates = vec![Dependency::Physical(renamed)];
                if parts.len() == 1 || parts != old {
                    candidates.push(dependency.clone());
                }
                candidates
            }
            other => vec![other.clone()],
        })
        .collect()
}
