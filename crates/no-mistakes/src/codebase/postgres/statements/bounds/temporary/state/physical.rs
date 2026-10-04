//! Permanent view dependency nodes share the request-local lifecycle state.
use super::{names::names_match, Dependency, State};
use std::collections::{BTreeMap, BTreeSet};

impl State {
    pub fn drop_physical(&mut self, mut dropped: BTreeSet<Vec<String>>, cascade: bool) -> bool {
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
            return false;
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
        true
    }

    pub fn rename_physical(
        &mut self,
        old: &[String],
        new: &str,
        targets: &BTreeSet<Vec<String>>,
    ) -> bool {
        if old.len() > 1 && self.physical_views.contains_key(old) {
            let mut target = old.to_vec();
            target.pop();
            target.push(new.to_owned());
            if self.physical_views.contains_key(&target) {
                // A known name collision rejects the entire rename in PostgreSQL.
                return false;
            }
        }
        let mut moved = BTreeMap::new();
        for (name, dependencies) in std::mem::take(&mut self.physical_views) {
            let dependencies = rename_dependencies(&dependencies, old, new, targets);
            let (name, original) = rename_identity(&name, old, new, targets);
            if let Some(original) = original {
                moved
                    .entry(original)
                    .or_insert_with(BTreeSet::new)
                    .extend(dependencies.iter().cloned());
            }
            moved
                .entry(name)
                .or_insert_with(BTreeSet::new)
                .extend(dependencies);
        }
        self.physical_views = moved;
        true
    }
}

pub(super) fn rename_dependencies(
    dependencies: &BTreeSet<Dependency>,
    old: &[String],
    new: &str,
    targets: &BTreeSet<Vec<String>>,
) -> BTreeSet<Dependency> {
    dependencies
        .iter()
        .flat_map(|dependency| match dependency {
            Dependency::Physical(parts) => {
                let (renamed, original) = rename_identity(parts, old, new, targets);
                std::iter::once(renamed)
                    .chain(original)
                    .map(Dependency::Physical)
                    .collect()
            }
            other => vec![other.clone()],
        })
        .collect()
}

// Ambiguous bare declarations and qualified namesakes keep their old candidate.
fn rename_identity(
    parts: &[String],
    old: &[String],
    new: &str,
    targets: &BTreeSet<Vec<String>>,
) -> (Vec<String>, Option<Vec<String>>) {
    if !targets.iter().any(|target| names_match(parts, target)) {
        return (parts.to_vec(), None);
    }
    let mut renamed = parts.to_vec();
    renamed.pop();
    renamed.push(new.to_owned());
    let original = (parts.len() == 1 || parts != old).then(|| parts.to_vec());
    (renamed, original)
}
