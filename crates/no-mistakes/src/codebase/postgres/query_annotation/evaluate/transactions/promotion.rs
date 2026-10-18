use super::{Cell, Checkpoint, Saved};
use crate::codebase::postgres::query_annotation::evaluate::{
    frame_arena::FrameCheckpoint, Environment, Evaluator,
};
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn transaction_frame_checkpoint(
        &self,
        cp: Checkpoint,
    ) -> FrameCheckpoint {
        self.transactions.frames[cp.0].scopes
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn transaction_argument_cutoff(
        &self,
        cp: Checkpoint,
    ) -> u64 {
        self.transactions.frames[cp.0].identities
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn promote_module_initial(
        &mut self,
        path: &Path,
        module: Environment,
        frames: &[Environment],
        arguments: &[u64],
        identities: &[u64],
    ) {
        let arguments = arguments
            .iter()
            .copied()
            .collect::<crate::fx::FxHashSet<_>>();
        let identities = identities
            .iter()
            .copied()
            .collect::<crate::fx::FxHashSet<_>>();
        for index in 0..self.transactions.frames.len() {
            if self.transactions.frames[index].root
                || self
                    .scopes
                    .is_before(module, self.transactions.frames[index].scopes)
            {
                // Completion only promotes modules first reserved inside this
                // checkpoint; an older partial module keeps its own before-image.
                continue;
            }
            let frames = frames
                .iter()
                .copied()
                .filter(|env| {
                    !self
                        .scopes
                        .is_before(*env, self.transactions.frames[index].scopes)
                })
                .collect::<crate::fx::FxHashSet<_>>();
            let rebased = self.transactions.frames[index]
                .old
                .keys()
                .filter(|cell| match cell {
                    Cell::Binding(env, _) | Cell::Fresh(env, _) => frames.contains(env),
                    Cell::Dense(id, _)
                    | Cell::Length(id)
                    | Cell::Extra(id, _)
                    | Cell::Deleted(id, _)
                    | Cell::Definite(id, _)
                    | Cell::Disconnected(id, _)
                    | Cell::Recreated(id, _) => arguments.contains(id),
                    Cell::Invalidated(id) | Cell::Builder(id) => identities.contains(id),
                    Cell::Module(module) => module == path,
                    Cell::Trust
                    | Cell::CallbackSeen(_)
                    | Cell::CallbackRunning(_)
                    | Cell::CallbackEnabled
                    | Cell::ExecutionsEnabled => false,
                })
                .cloned()
                .collect::<Vec<_>>();
            for cell in rebased {
                let current = self.read_cell(&cell);
                self.transactions.frames[index].rebased_values_cloned +=
                    u64::from(matches!(current, Saved::Value(Some(_))));
                self.transactions.frames[index].old.insert(cell, current);
            }
            let frame = &mut self.transactions.frames[index];
            frame.promoted_frames.extend(frames.iter().copied());
            frame.promoted_ids.extend(identities.iter().copied());
            self.rebuild_transaction_pins(Checkpoint(index));
        }
    }
}
