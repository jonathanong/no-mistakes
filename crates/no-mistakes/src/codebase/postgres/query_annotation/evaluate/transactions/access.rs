use super::{Cell, Checkpoint, Frame, Saved};
use crate::codebase::postgres::query_annotation::evaluate::Evaluator;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn begin_transaction(
        &mut self,
    ) -> Checkpoint {
        self.begin_checkpoint(false)
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn begin_root_transaction(
        &mut self,
    ) -> Checkpoint {
        self.begin_checkpoint(true)
    }
    fn begin_checkpoint(&mut self, root: bool) -> Checkpoint {
        let checkpoint = Checkpoint(self.transactions.frames.len());
        self.transactions.frames.push(Frame {
            scopes: self.scopes.checkpoint(),
            identities: self.next_builder,
            old: Default::default(),
            writes: Default::default(),
            reference_pins: Default::default(),
            root,
            recorded: 0,
            values_cloned: 0,
            rebased_values_cloned: 0,
            promoted_frames: Default::default(),
            promoted_ids: Default::default(),
        });
        checkpoint
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn save_cell(
        &mut self,
        cell: Cell,
    ) {
        if self.transactions.frames.is_empty() {
            return;
        }
        let mut missing = Vec::new();
        for (index, frame) in self.transactions.frames.iter_mut().enumerate() {
            frame.writes.insert(cell.clone());
            let incoming = match &cell {
                Cell::Binding(env, _) | Cell::Fresh(env, _) => {
                    self.scopes.is_before(*env, frame.scopes) || frame.promoted_frames.contains(env)
                }
                Cell::Trust
                | Cell::Module(_)
                | Cell::CallbackSeen(_)
                | Cell::CallbackRunning(_)
                | Cell::CallbackEnabled
                | Cell::ExecutionsEnabled => true,
                Cell::Dense(id, _)
                | Cell::Length(id)
                | Cell::Extra(id, _)
                | Cell::Invalidated(id)
                | Cell::Builder(id)
                | Cell::Deleted(id, _)
                | Cell::Definite(id, _)
                | Cell::Disconnected(id, _)
                | Cell::Recreated(id, _) => {
                    *id < frame.identities || frame.promoted_ids.contains(id)
                }
            };
            if incoming && !frame.old.contains_key(&cell) {
                missing.push(index);
            }
        }
        if missing.is_empty() {
            return;
        }
        let mut before = Some(self.read_cell(&cell));
        let last = missing.len() - 1;
        for (position, index) in missing.into_iter().enumerate() {
            let saved = if position == last {
                before.take().unwrap()
            } else {
                before.as_ref().unwrap().clone()
            };
            self.pin_transaction_cell(Checkpoint(index), &cell, &saved);
            let frame = &mut self.transactions.frames[index];
            frame.recorded += 1;
            frame.values_cloned += u64::from(matches!(saved, Saved::Value(Some(_))));
            self.transactions.frames[index]
                .old
                .insert(cell.clone(), saved);
        }
    }
}
