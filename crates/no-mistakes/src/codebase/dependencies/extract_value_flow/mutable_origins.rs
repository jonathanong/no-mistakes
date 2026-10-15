use super::*;
use std::collections::BTreeMap;
#[path = "mutable_origin_data.rs"]
mod data;
#[path = "mutable_origin_pending.rs"]
mod pending;
#[path = "mutable_origin_phases.rs"]
mod phases;
#[path = "mutable_origin_records.rs"]
mod records;
#[path = "mutable_origin_single.rs"]
pub(super) mod single;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct RootVersion {
    context: Option<CallableId>,
    selected: Option<usize>,
    generation: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct RootIdentity {
    context: Option<CallableId>,
    selected: Option<usize>,
}
impl RootVersion {
    pub(super) fn identity(self) -> RootIdentity {
        RootIdentity {
            context: self.context,
            selected: self.selected,
        }
    }

    pub(super) fn same_selected(&self, other: &Self) -> bool {
        self.context == other.context && self.selected == other.selected
    }
}
#[path = "mutable_origin_possible.rs"]
mod possible;
#[path = "mutable_origin_snapshots.rs"]
mod snapshots;
#[derive(Default)]
struct RootStates {
    snapshots: snapshots::RootSnapshots,
    possible: possible::PossibleOrigins,
    #[cfg(feature = "test-instrumentation")]
    insertion_order: [usize; 3],
}
#[derive(Default)]
struct RootOrigins {
    patterns: Option<Box<mutable_patterns::PatternRoots>>,
    owners: FxHashMap<Option<CallableId>, RootStates>,
    generation: usize,
    #[cfg(feature = "test-instrumentation")]
    predecessor_queries: usize,
}
pub(super) enum Origin {
    SingleNamedField(single::SingleNamedField),
    Value(ValueExpression),
    Either(usize, usize),
}
/// Indexed lexical-root facts only. Members still use canonical receiver histories.
#[derive(Default)]
pub(in super::super) struct ReceiverBindingOrigins {
    roots: FxHashMap<usize, FxHashMap<String, RootOrigins>>,
    arena: Vec<Origin>,
    names: Option<Box<single::Names>>,
    #[cfg(feature = "test-instrumentation")]
    lookups: std::cell::Cell<usize>,
    #[cfg(feature = "test-instrumentation")]
    visits: std::cell::Cell<usize>,
    #[cfg(feature = "test-instrumentation")]
    spills: std::cell::Cell<usize>,
    #[cfg(feature = "test-instrumentation")]
    membership_probes: std::cell::Cell<usize>,
    #[cfg(feature = "test-instrumentation")]
    owned_keys: usize,
    #[cfg(feature = "test-instrumentation")]
    snapshot_queries: std::cell::Cell<[usize; 3]>,
}
impl ReceiverBindingOrigins {
    pub(super) fn watermark(&self) -> usize {
        self.arena.len()
    }
    pub(super) fn contains(&self, scope: usize, name: &str) -> bool {
        #[cfg(all(test, feature = "test-instrumentation"))]
        fusion_tests::QUERIES.with(|n| n.set(n.get() + 1));
        #[cfg(feature = "test-instrumentation")]
        self.membership_probes.set(self.membership_probes.get() + 1);
        self.roots
            .get(&scope)
            .is_some_and(|names| names.contains_key(name))
    }
    pub(super) fn bindings(&self) -> impl Iterator<Item = (usize, &str)> {
        self.roots
            .iter()
            .flat_map(|(scope, names)| names.keys().map(move |name| (*scope, name.as_str())))
    }
    pub(super) fn version(
        &self,
        key: &(usize, String),
        read: nested::RootRead,
    ) -> Option<RootVersion> {
        #[cfg(all(test, feature = "test-instrumentation"))]
        fusion_tests::QUERIES.with(|n| n.set(n.get() + 1));
        let root = self.roots.get(&key.0)?.get(&key.1)?;
        #[cfg(feature = "test-instrumentation")]
        self.lookups.set(self.lookups.get() + 1);
        let selected = root
            .owners
            .get(&read.1)
            .and_then(|states| {
                #[cfg(feature = "test-instrumentation")]
                {
                    let [calls, latest, searches] = self.snapshot_queries.get();
                    self.snapshot_queries.set([calls + 1, latest, searches]);
                    states.snapshots.at_counted(read.0, &self.snapshot_queries)
                }
                #[cfg(not(feature = "test-instrumentation"))]
                states.snapshots.at(read.0)
            })
            .map(|(_, id)| *id);
        Some(RootVersion {
            context: read.1,
            selected: root
                .patterns
                .as_ref()
                .and_then(|patterns| patterns.selected(read))
                .unwrap_or(selected),
            generation: root.generation,
        })
    }
    pub(super) fn append_references<R: nested::RootCoordinate>(
        &self,
        key: &(usize, String),
        coordinates: (&[Option<String>], &[nested::ReceiverRead], nested::RootRead),
        output: &mut impl Extend<nested::ReceiverWrite<R>>,
        foreign_before: usize,
        version: RootVersion,
    ) {
        let Some(root) = self.roots.get(&key.0).and_then(|names| names.get(&key.1)) else {
            return;
        };
        let (path, reads, root_read) = coordinates;
        let mut pending = pending::PendingOrigins::default();
        let phase_selected = root
            .patterns
            .as_ref()
            .and_then(|patterns| patterns.selected(root_read));
        if phase_selected.is_some() {
            pending.extend(version.selected.map(|id| (id, false)));
        }
        for (owner, states) in &root.owners {
            if root_read.1 != *owner {
                // Normal leaves append in arena order. Later foreign bodies
                // cannot force replay to walk a suffix beyond this acquisition.
                pending.extend(states.possible.before(foreign_before).map(|id| (id, true)));
            } else if phase_selected.is_none() {
                pending.extend(version.selected.map(|id| (id, false)));
            }
        }
        // Arena links share alternatives without recursive traversal or drop.
        while let Some((id, foreign)) = pending.pop() {
            #[cfg(feature = "test-instrumentation")]
            self.visits.set(self.visits.get() + 1);
            match &self.arena[id] {
                Origin::Value(value) => {
                    if !foreign || id < foreign_before {
                        nested::write_receiver_references(value, path, reads, output)
                    }
                }
                Origin::SingleNamedField(value) => {
                    if !foreign || id < foreign_before {
                        self.append_single(value, path, reads, output);
                    }
                }
                Origin::Either(left, right) => {
                    pending.extend([(*left, foreign), (*right, foreign)])
                }
            }
        }
        #[cfg(feature = "test-instrumentation")]
        self.spills
            .set(self.spills.get() + usize::from(pending.spilled()));
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "receiver_lookup_fusion_tests.rs"]
pub(super) mod fusion_tests;
