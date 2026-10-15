use super::*;
use phase_catalog::PhaseCatalog;
use phase_walk::PhaseWrite;
use receiver_histories::ReceiverHistories;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
#[derive(Default)]
struct PhaseNode {
    children: FxHashMap<String, PhaseNode>,
    writes: BTreeSet<u32>,
    possible_original: BTreeSet<u32>,
    dynamic: BTreeSet<u32>,
}
pub(super) struct ActiveReferencePhases {
    catalog: Arc<PhaseCatalog>,
    rhs_end: u32,
    generation: Option<usize>,
    owner: Option<CallableId>,
    roots: FxHashMap<usize, PhaseNode>,
    pending: FxHashMap<u32, Vec<(String, ValueMutationPath, u32, bool)>>,
    unresolved: BTreeMap<u32, usize>,
    write_phases: FxHashMap<u32, u32>,
    effects: Option<BTreeSet<(u32, u32)>>,
    pub(super) inert_rhs: bool,
    #[cfg(feature = "test-instrumentation")]
    stats: active_phase_stats::PhaseStats,
}
#[derive(Clone, Copy)]
pub(super) struct PhaseCursor<'a> {
    phases: &'a ActiveReferencePhases,
    node: Option<&'a PhaseNode>,
    trusted: bool,
}
impl ActiveReferencePhases {
    pub(super) fn new(
        coordinates: (&Arc<PhaseCatalog>, u32),
        writes: &[PhaseWrite],
        scopes: Option<&FxHashMap<u32, usize>>,
        histories: &ReceiverHistories,
        owner: Option<CallableId>,
        inert_rhs: bool,
        proof_events: (Option<BTreeSet<(u32, u32)>>, Option<BTreeSet<u32>>),
    ) -> Self {
        let (catalog, rhs_end) = coordinates;
        let (effects, fresh_phases) = proof_events;
        let mut result = Self {
            catalog: Arc::clone(catalog),
            rhs_end,
            generation: histories.component_generation(),
            owner,
            roots: fx_map(),
            pending: fx_map(),
            unresolved: BTreeMap::new(),
            write_phases: fx_map(),
            effects,
            inert_rhs,
            #[cfg(feature = "test-instrumentation")]
            stats: Default::default(),
        };
        #[cfg(feature = "test-instrumentation")]
        {
            result.stats.fresh_entries = fresh_phases.as_ref().map_or(0, BTreeSet::len);
        }
        for write in writes {
            let fresh = fresh_phases
                .as_ref()
                .is_some_and(|phases| phases.contains(&write.phase));
            #[cfg(feature = "test-instrumentation")]
            {
                result.stats.path_tokens += write.path.len();
            }
            result.write_phases.insert(write.target, write.phase);
            if let Some(scope) = scopes.and_then(|scopes| scopes.get(&write.target)) {
                result.insert(
                    histories.proof_component(*scope, write.name.split('.').next().unwrap()),
                    &write.path,
                    write.phase,
                    fresh,
                );
            } else {
                #[cfg(feature = "test-instrumentation")]
                {
                    result.stats.pending_tokens += write.path.len();
                }
                *result.unresolved.entry(write.phase).or_default() += 1;
                result.pending.entry(write.target).or_default().push((
                    write.name.clone(),
                    write.path.clone(),
                    write.phase,
                    fresh,
                ));
            }
        }
        result
    }
    pub(super) fn capture_phase(&self, target: Option<u32>) -> Option<(u32, u32)> {
        Some((self.catalog.start, *self.write_phases.get(&target?)?))
    }
    fn insert(
        &mut self,
        component: Option<usize>,
        path: &[Option<String>],
        phase: u32,
        fresh: bool,
    ) {
        let Some(component) = component else {
            return;
        };
        #[cfg(feature = "test-instrumentation")]
        let nodes = &mut self.stats.nodes;
        let mut node = self.roots.entry(component).or_insert_with(|| {
            #[cfg(feature = "test-instrumentation")]
            {
                *nodes += 1;
            }
            PhaseNode::default()
        });
        for member in path {
            let Some(member) = member else {
                let inserted = node.dynamic.insert(phase);
                #[cfg(feature = "test-instrumentation")]
                {
                    self.stats.write_entries += usize::from(inserted);
                }
                let _ = inserted;
                return;
            };
            node = node.children.entry(member.clone()).or_insert_with(|| {
                #[cfg(feature = "test-instrumentation")]
                {
                    *nodes += 1;
                }
                PhaseNode::default()
            });
        }
        let inserted = node.writes.insert(phase);
        #[cfg(feature = "test-instrumentation")]
        {
            self.stats.write_entries += usize::from(inserted);
        }
        let _ = inserted;
        let original = !fresh && node.possible_original.insert(phase);
        #[cfg(feature = "test-instrumentation")]
        {
            self.stats.original_entries += usize::from(original);
        }
        let _ = original;
    }
    pub(super) fn resolve(&mut self, target: u32, scope: usize, histories: &ReceiverHistories) {
        for (name, path, phase, fresh) in self.pending.remove(&target).into_iter().flatten() {
            #[cfg(feature = "test-instrumentation")]
            {
                self.stats.pending_tokens -= path.len();
            }
            if let Some(count) = self.unresolved.get_mut(&phase) {
                *count -= 1;
                if *count == 0 {
                    self.unresolved.remove(&phase);
                }
            }
            self.insert(
                histories.proof_component(scope, name.split('.').next().unwrap()),
                &path,
                phase,
                fresh,
            );
        }
    }
    pub(super) fn cursor<'a>(
        &'a self,
        histories: &ReceiverHistories,
        key: &(usize, String),
        owner: Option<CallableId>,
    ) -> PhaseCursor<'a> {
        PhaseCursor {
            phases: self,
            node: histories
                .proof_component(key.0, &key.1)
                .and_then(|id| self.roots.get(&id)),
            trusted: self.generation.is_some()
                && self.generation == histories.component_generation()
                && self.owner == owner,
        }
    }
}
#[cfg(feature = "test-instrumentation")]
#[path = "active_phase_stats.rs"]
mod active_phase_stats;
#[path = "active_phase_cursor.rs"]
mod cursor;
