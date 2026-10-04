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
    ) -> Option<BTreeSet<Vec<String>>> {
        let collisions = self.rename_collisions(new, targets);
        if collisions.contains(old) {
            // An exact known collision rejects the whole qualified rename.
            return None;
        }
        let mut moved = BTreeMap::new();
        for (name, dependencies) in std::mem::take(&mut self.physical_views) {
            let dependencies = rename_dependencies(&dependencies, old, new, targets, &collisions);
            let (name, original) = rename_identity(&name, old, new, targets, &collisions);
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
        Some(collisions)
    }

    fn rename_collisions(
        &self,
        new: &str,
        targets: &BTreeSet<Vec<String>>,
    ) -> BTreeSet<Vec<String>> {
        self.physical_views
            .keys()
            .filter(|source| {
                if source.len() <= 1 || !targets.iter().any(|target| names_match(source, target)) {
                    return false;
                }
                let mut destination = (*source).clone();
                destination.pop();
                destination.push(new.to_owned());
                self.physical_views.contains_key(&destination)
            })
            .cloned()
            .collect()
    }
}

pub(super) fn rename_dependencies(
    dependencies: &BTreeSet<Dependency>,
    old: &[String],
    new: &str,
    targets: &BTreeSet<Vec<String>>,
    collisions: &BTreeSet<Vec<String>>,
) -> BTreeSet<Dependency> {
    dependencies
        .iter()
        .flat_map(|dependency| match dependency {
            Dependency::Physical(parts) => {
                let (renamed, original) = rename_identity(parts, old, new, targets, collisions);
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
    collisions: &BTreeSet<Vec<String>>,
) -> (Vec<String>, Option<Vec<String>>) {
    if collisions.contains(parts) || !targets.iter().any(|target| names_match(parts, target)) {
        return (parts.to_vec(), None);
    }
    let mut renamed = parts.to_vec();
    renamed.pop();
    renamed.push(new.to_owned());
    let original = (parts.len() == 1 || parts != old).then(|| parts.to_vec());
    (renamed, original)
}
