//! Rollback consumes only written before-images; roots never build after-images.
use super::{Cell, Change, Checkpoint, Delta, Saved};
use crate::codebase::postgres::query_annotation::evaluate::Evaluator;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

fn late(cell: &Cell) -> bool {
    matches!(
        cell,
        Cell::Length(_) | Cell::CallbackEnabled | Cell::ExecutionsEnabled
    )
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    fn take_checkpoint(&mut self, cp: Checkpoint) -> FxHashMap<Cell, Saved> {
        assert_eq!(self.transactions.frames.len(), cp.0 + 1);
        std::mem::take(&mut self.transactions.frames[cp.0].old)
    }
    fn restore_checkpoint<'a>(
        &mut self,
        entries: impl Iterator<Item = (&'a Cell, &'a Saved)> + Clone,
    ) {
        let (mut restored, mut cloned) = (0, 0);
        for (cell, before) in entries.clone().filter(|(cell, _)| !late(cell)) {
            self.restore_cell(cell, before);
            restored += 1;
            cloned += u64::from(matches!(before, Saved::Value(Some(_))));
        }
        for (cell, before) in entries.clone().filter(|(cell, _)| late(cell)) {
            self.restore_cell(cell, before);
            restored += 1;
        }
        // Primary masks and proofs must all be restored before derived edges.
        for (cell, _) in entries {
            self.restore_transaction_index(cell);
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.undo_entries_restored", restored);
            observer.increment("postgres.annotation.undo_restore_values_cloned", cloned);
        }
    }
    fn finish_checkpoint(&mut self, cp: Checkpoint) {
        self.release_transaction_pins(cp);
        let frame = self.transactions.frames.pop().unwrap();
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.undo_entries_recorded", frame.recorded);
            observer.increment(
                "postgres.annotation.undo_values_cloned",
                frame.values_cloned,
            );
            observer.increment(
                "postgres.annotation.undo_rebased_values_cloned",
                frame.rebased_values_cloned,
            );
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn finish_transaction(
        &mut self,
        cp: Checkpoint,
    ) -> Delta {
        let changes = self
            .take_checkpoint(cp)
            .into_iter()
            .map(|(cell, before)| {
                let after = self.read_cell(&cell);
                Change {
                    cell,
                    before,
                    after,
                }
            })
            .collect::<Vec<_>>();
        let pins = self.pin_delta_values(&changes);
        self.restore_checkpoint(changes.iter().map(|change| (&change.cell, &change.before)));
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(
                "postgres.annotation.undo_after_values_cloned",
                changes
                    .iter()
                    .filter(|change| matches!(change.after, Saved::Value(Some(_))))
                    .count() as u64,
            );
        }
        self.finish_checkpoint(cp);
        Delta { changes, pins }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn finish_root_transaction(
        &mut self,
        cp: Checkpoint,
    ) {
        self.discard_root_allocations(
            self.transaction_frame_checkpoint(cp),
            self.transaction_argument_cutoff(cp),
        );
        let old = self.take_checkpoint(cp);
        self.restore_checkpoint(old.iter());
        self.finish_checkpoint(cp);
        self.reclaim_dead_state();
        self.assert_checkpoint_indexes();
        if self.transactions.frames.is_empty() && self.active_module_initials.is_empty() {
            self.scopes.clear_allocation_log();
        }
    }
}
