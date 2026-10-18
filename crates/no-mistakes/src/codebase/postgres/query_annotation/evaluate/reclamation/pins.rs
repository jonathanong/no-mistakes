use super::super::{
    transactions::{Change, Checkpoint, Saved},
    Evaluator,
};
use super::*;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn identity_is_pinned(&self, id: u64) -> bool {
        self.transactions.reclamation.pins.contains_key(&id)
    }
    pub(in super::super) fn pinned_identity_roots(
        &self,
        cutoff: u64,
    ) -> Vec<super::super::temporaries::TransientRoot> {
        self.transactions
            .reclamation
            .pins
            .range(cutoff..)
            .map(|(id, _)| {
                if self.argument_objects.contains_key(id) {
                    super::super::temporaries::TransientRoot::Argument(*id)
                } else {
                    super::super::temporaries::TransientRoot::Builder(*id)
                }
            })
            .collect()
    }
    fn cell_pins(&self, cell: &Cell, saved: &Saved) -> FxHashSet<u64> {
        let mut pins = FxHashSet::default();
        if let Saved::Value(Some(value)) = saved {
            pins.extend(References::value(value).identities());
        }
        match cell {
            Cell::Dense(id, _)
            | Cell::Extra(id, _)
            | Cell::Length(id)
            | Cell::Builder(id)
            | Cell::Invalidated(id)
            | Cell::Deleted(id, _)
            | Cell::Definite(id, _)
            | Cell::Disconnected(id, _)
            | Cell::Recreated(id, _) => {
                pins.insert(*id);
            }
            _ => {}
        }
        pins
    }
    pub(in super::super) fn pin_transaction_cell(
        &mut self,
        cp: Checkpoint,
        cell: &Cell,
        saved: &Saved,
    ) {
        let pins = self.cell_pins(cell, saved);
        for id in pins {
            if self.transactions.frames[cp.0].reference_pins.insert(id) {
                self.transactions.reclamation.pin(id);
            }
        }
    }
    pub(in super::super) fn release_transaction_pins(&mut self, cp: Checkpoint) {
        let pins = std::mem::take(&mut self.transactions.frames[cp.0].reference_pins);
        for id in pins {
            self.transactions.reclamation.unpin(id);
        }
    }
    pub(in super::super) fn rebuild_transaction_pins(&mut self, cp: Checkpoint) {
        let frame = &self.transactions.frames[cp.0];
        let pins = frame
            .old
            .iter()
            .flat_map(|(cell, saved)| self.cell_pins(cell, saved))
            .collect::<FxHashSet<_>>();
        let previous = std::mem::replace(&mut self.transactions.frames[cp.0].reference_pins, pins);
        let current = &self.transactions.frames[cp.0].reference_pins;
        for id in previous.difference(current) {
            self.transactions.reclamation.unpin(*id);
        }
        for id in current.difference(&previous) {
            self.transactions.reclamation.pin(*id);
        }
    }
    pub(in super::super) fn pin_value_roots(
        &mut self,
        values: &[super::super::Value],
        roots: &[super::super::temporaries::TransientRoot],
    ) -> FxHashSet<u64> {
        let mut pins = values
            .iter()
            .flat_map(|value| References::value(value).identities().collect::<Vec<_>>())
            .collect::<FxHashSet<_>>();
        for root in roots {
            match root {
                super::super::temporaries::TransientRoot::Argument(id)
                | super::super::temporaries::TransientRoot::Builder(id) => {
                    pins.insert(*id);
                }
                super::super::temporaries::TransientRoot::Environment(_) => {}
            }
        }
        for id in &pins {
            self.transactions.reclamation.pin(*id);
        }
        pins
    }
    pub(in super::super) fn pin_delta_values(&mut self, changes: &[Change]) -> FxHashSet<u64> {
        let pins = changes
            .iter()
            .flat_map(|change| self.cell_pins(&change.cell, &change.after))
            .collect::<FxHashSet<_>>();
        for id in &pins {
            self.transactions.reclamation.pin(*id);
        }
        pins
    }
    pub(in super::super) fn release_delta_pins(&mut self, pins: FxHashSet<u64>) {
        for id in pins {
            self.transactions.reclamation.unpin(id);
        }
    }
    pub(in super::super) fn pin_module_identities(&mut self, ids: &[u64]) {
        for id in ids {
            self.transactions.reclamation.pin(*id);
        }
    }
    pub(in super::super) fn unpin_module_identities(&mut self, ids: &[u64]) {
        for id in ids {
            self.transactions.reclamation.unpin(*id);
        }
    }
}
