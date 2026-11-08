use std::collections::{BTreeMap, BTreeSet};

pub(crate) type Identity = (Vec<u32>, usize);
pub(crate) type Colors<K> = BTreeMap<K, BTreeMap<Identity, usize>>;

pub(crate) struct SiteIdentities<K> {
    scopes: BTreeMap<K, BTreeMap<usize, BTreeSet<Identity>>>,
}

impl<K> Default for SiteIdentities<K> {
    fn default() -> Self {
        Self {
            scopes: BTreeMap::new(),
        }
    }
}

#[derive(Default)]
struct Used {
    colors: BTreeSet<usize>,
    next: usize,
}

impl<K: Ord> SiteIdentities<K> {
    pub(crate) fn record(&mut self, scope: K, projection: usize, identity: Identity) {
        self.scopes
            .entry(scope)
            .or_default()
            .entry(projection)
            .or_default()
            .insert(identity);
    }

    pub(crate) fn finish(self) -> Colors<K> {
        self.finish_with(|| {})
    }

    fn finish_with(self, mut probe: impl FnMut()) -> Colors<K> {
        self.scopes
            .into_iter()
            .map(|(scope, projections)| {
                let mut memberships = BTreeMap::<Identity, Vec<usize>>::new();
                let mut used = BTreeMap::<usize, Used>::new();
                for (projection, identities) in projections {
                    used.insert(projection, Used::default());
                    for identity in identities {
                        memberships.entry(identity).or_default().push(projection);
                    }
                }
                let mut colors = BTreeMap::new();
                let mut candidates = BTreeMap::<Vec<usize>, usize>::new();
                for (identity, projections) in memberships {
                    let mut color = projections
                        .iter()
                        .map(|projection| used[projection].next)
                        .max()
                        .unwrap()
                        .max(candidates.get(&projections).copied().unwrap_or_default());
                    while projections
                        .iter()
                        .any(|projection| used[projection].colors.contains(&color))
                    {
                        probe();
                        color += 1;
                    }
                    candidates.insert(projections.clone(), color + 1);
                    for projection in projections {
                        let used = used.get_mut(&projection).unwrap();
                        used.colors.insert(color);
                        while used.colors.contains(&used.next) {
                            used.next += 1;
                        }
                    }
                    colors.insert(identity, color);
                }
                (scope, colors)
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "site_identity_tests.rs"]
mod tests;
