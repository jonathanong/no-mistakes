use super::super::Evaluator;
use super::*;
use std::path::{Path, PathBuf};
impl Reclamation {
    fn dead_component(
        &self,
        starts: Vec<u64>,
        exists: impl Fn(u64) -> bool,
    ) -> (FxHashSet<u64>, usize) {
        let mut component = FxHashSet::default();
        let mut pending = starts;
        while let Some(id) = pending.pop() {
            if !exists(id) || !component.insert(id) {
                continue;
            }
            pending.extend(self.targets(id));
        }
        let mut internal = FxHashMap::<u64, usize>::default();
        for id in &component {
            for location in self.owners.get(&Owner::Argument(*id)).into_iter().flatten() {
                for target in &self.locations[location].arguments {
                    *internal.entry(*target).or_default() += 1;
                }
            }
        }
        let mut pending = component
            .iter()
            .filter(|id| {
                self.pins.contains_key(id)
                    || self.incoming.get(id).copied().unwrap_or(0)
                        > internal.get(id).copied().unwrap_or(0)
            })
            .copied()
            .collect::<Vec<_>>();
        let mut live = FxHashSet::default();
        while let Some(id) = pending.pop() {
            if !live.insert(id) {
                continue;
            }
            pending.extend(self.targets(id).filter(|target| component.contains(target)));
        }
        let visited = component.len();
        component.retain(|id| !live.contains(id));
        (component, visited)
    }
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn reclaim_dead_state(&mut self) {
        let transient = self.transient_roots.clone();
        let transient_pins = self.pin_value_roots(&[], &transient);
        let observer = crate::diagnostics::current();
        let mut candidates = std::mem::take(&mut self.transactions.reclamation.candidates)
            .into_iter()
            .collect::<Vec<_>>();
        candidates.retain(|id| {
            self.argument_objects.contains_key(id)
                && !self.transactions.reclamation.roots.contains_key(id)
                && !self.transactions.reclamation.pins.contains_key(id)
        });
        candidates.sort_unstable();
        let (dead, visits) = self
            .transactions
            .reclamation
            .dead_component(candidates, |id| self.argument_objects.contains_key(&id));
        if let Some(observer) = &observer {
            observer.increment(
                "postgres.annotation.old_argument_trial_visits",
                visits as u64,
            );
            observer.increment(
                "postgres.annotation.old_arguments_reclaimed",
                dead.len() as u64,
            );
        }
        for id in dead {
            self.drop_argument_state(id);
        }
        // Removing dead owners only queues their forward targets. The batched
        // component already visited every such argument; surviving targets have
        // an external root/pin, so repeating that traversal would add no evidence.
        self.transactions.reclamation.candidates.clear();
        let ids = std::mem::take(&mut self.transactions.reclamation.builder_candidates);
        for id in ids {
            let graph = &self.transactions.reclamation;
            if !graph.holders.contains_key(&id)
                && !graph.pins.contains_key(&id)
                && !self.argument_objects.contains_key(&id)
            {
                self.builder_updates.remove(&id);
                self.invalidated_builders.remove(&id);
                self.transactions.reclamation.masks.remove(&id);
            }
        }
        self.release_delta_pins(transient_pins);
        self.transactions.reclamation.report_reference_metadata();
        self.transactions.reclamation.shrink_reference_storage();
    }
    pub(in super::super) fn drop_argument_state(&mut self, id: u64) {
        self.transactions
            .reclamation
            .remove_owner(Owner::Argument(id));
        if let Some(masks) = self.transactions.reclamation.masks.remove(&id) {
            for cell in masks {
                match cell {
                    Cell::Deleted(id, index) => {
                        self.deleted_argument_slots.remove(&(id, index));
                    }
                    Cell::Definite(id, index) => {
                        self.definite_deleted_argument_slots.remove(&(id, index));
                    }
                    Cell::Disconnected(id, index) => {
                        self.disconnected_argument_slots.remove(&(id, index));
                    }
                    Cell::Recreated(id, index) => {
                        self.recreated_argument_slots.remove(&(id, index));
                    }
                    Cell::Invalidated(id) => {
                        self.invalidated_builders.remove(&id);
                    }
                    _ => unreachable!("identity mask"),
                }
            }
        }
        self.argument_objects.remove(&id);
        self.argument_extra_slots.remove(&id);
        self.builder_updates.remove(&id);
        self.invalidated_builders.remove(&id);
    }
}
