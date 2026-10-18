use super::*;

#[path = "assigned_foreign_providers.rs"]
mod foreign_providers;
#[path = "assigned_receiver_record.rs"]
mod record;
#[path = "assigned_receiver_timeline.rs"]
mod timeline;
use timeline::Timeline;

/// Extraction-only RHS prefixes share ReceiverHistories component ownership.
/// Each timeline is qualified by the acquired ancestor identities, so a saved
/// container cannot read a descendant store made on its replacement.
#[derive(Default)]
pub(super) struct AssignedReceiverOrigins {
    roots: FxHashMap<usize, FxHashMap<Option<mutable_origins::RootIdentity>, Prefixes>>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) queries: std::cell::Cell<usize>,
}
type Prefixes = std::collections::BTreeMap<ValueMutationPath, Owners>;
type Owners = FxHashMap<Option<CallableId>, FxHashMap<Vec<Option<u32>>, Timeline>>;
pub(super) struct Store<'a> {
    pub(super) owner: Option<CallableId>,
    pub(super) reads: &'a [nested::ReceiverRead],
    pub(super) completed: u32,
    pub(super) complete: bool,
    pub(super) stable_alias: bool,
    pub(super) capture: identity_metadata::FreshCapture,
    pub(super) root_identity: Option<mutable_origins::RootIdentity>,
    pub(super) parent_identities: Vec<Option<u32>>,
}
struct Origin {
    value: ValueExpression,
    complete: bool,
    identity: Option<u32>,
    capture: identity_metadata::FreshCapture,
}
pub(super) type Receipts = Vec<(usize, u32, Option<CallableId>)>;
pub(super) struct Selection<'a> {
    pub(super) values: Vec<(&'a ValueExpression, usize)>,
    pub(super) receipts: Receipts,
    pub(super) complete: bool,
    pub(super) capture: Option<&'a identity_metadata::FreshCapture>,
}
impl Selection<'_> {
    pub(super) fn append_references<R: nested::RootCoordinate>(
        &self,
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        output: &mut impl Extend<nested::ReceiverWrite<R>>,
    ) {
        for (value, prefix) in &self.values {
            nested::write_receiver_references(value, &path[*prefix..], &reads[*prefix..], output);
        }
    }

    pub(super) fn captured(self) -> Option<(usize, identity_metadata::FreshCapture)> {
        self.complete.then(|| {
            (
                self.values[0].1,
                self.capture.expect("exact RHS capture").clone(),
            )
        })
    }
}
impl AssignedReceiverOrigins {
    pub(super) fn merge(&mut self, into: usize, from: usize) {
        if let Some(versions) = self.roots.remove(&from) {
            let target = self.roots.entry(into).or_default();
            for (version, prefixes) in versions {
                let target = target.entry(version).or_default();
                for (path, owners) in prefixes {
                    let target = target.entry(path).or_default();
                    for (owner, identities) in owners {
                        let target = target.entry(owner).or_default();
                        for (identity, mut events) in identities {
                            target.entry(identity).or_default().append(&mut events);
                        }
                    }
                }
            }
        }
    }
    pub(super) fn interested(
        &self,
        root: usize,
        path: &[Option<String>],
        version: Option<mutable_origins::RootIdentity>,
    ) -> bool {
        self.roots
            .get(&root)
            .and_then(|versions| versions.get(&version))
            .is_some_and(|prefixes| {
                (1..=path.len()).any(|end| prefixes.contains_key(&path[..end]))
                    || prefixes
                        .range(path.to_vec()..)
                        .next()
                        .is_some_and(|(candidate, _)| candidate.starts_with(path))
            })
    }
    fn identities(
        &self,
        root: usize,
        version: Option<mutable_origins::RootIdentity>,
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        canonical: &[Option<u32>],
    ) -> Vec<Option<u32>> {
        let mut identities = Vec::with_capacity(path.len().saturating_sub(1));
        for end in 1..path.len() {
            #[cfg(feature = "test-instrumentation")]
            self.queries.set(self.queries.get() + 1);
            let event = reads[end - 1].and_then(|(at, owner)| {
                self.roots
                    .get(&root)?
                    .get(&version)?
                    .get(&path[..end])?
                    .get(&owner)?
                    .get(&identities)?
                    .events
                    .range(..=at)
                    .next_back()
                    .and_then(|(_, origin)| origin.identity)
            });
            identities.push(event.or(canonical[end - 1]));
        }
        identities
    }
    pub(super) fn selected(
        &self,
        root: usize,
        version: Option<mutable_origins::RootIdentity>,
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        canonical: &[Option<u32>],
    ) -> Option<Selection<'_>> {
        let prefixes = self.roots.get(&root)?.get(&version)?;
        let identities = self.identities(root, version, path, reads, canonical);
        let mut selected: Option<(&u32, &Origin, usize)> = None;
        let mut possible = Vec::new();
        let mut ambiguous = false;
        let mut receipts = Vec::new();
        for end in 1..=path.len() {
            #[cfg(feature = "test-instrumentation")]
            self.queries.set(self.queries.get() + 1);
            let Some((at, owner)) = reads[end - 1] else {
                continue;
            };
            let Some(events) = prefixes
                .get(&path[..end])
                .and_then(|owners| owners.get(&owner))
                .and_then(|identities_map| identities_map.get(&identities[..end - 1]))
            else {
                continue;
            };
            if let Some((completed, origin)) = events.events.range(..=at).next_back() {
                receipts.push((end, *completed, owner));
                if !origin.complete || ambiguous {
                    if !ambiguous {
                        if let Some((_, previous, prefix)) = selected.take() {
                            possible.push((&previous.value, prefix));
                        }
                    }
                    ambiguous = true;
                    possible.push((&origin.value, end));
                } else if selected.is_none_or(|(previous, _, _)| completed > previous) {
                    selected = Some((completed, origin, end));
                }
            }
        }
        if ambiguous {
            Some(Selection {
                values: possible,
                receipts,
                complete: false,
                capture: None,
            })
        } else {
            selected.map(|(_, origin, end)| Selection {
                values: vec![(&origin.value, end)],
                receipts,
                complete: true,
                capture: Some(&origin.capture),
            })
        }
    }
}
