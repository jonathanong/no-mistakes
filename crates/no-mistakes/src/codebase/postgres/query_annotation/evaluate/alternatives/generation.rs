//! Arm collection visits only post-checkpoint allocations and remembered writes.
use super::super::{
    frame_arena::FrameCheckpoint, temporaries::TransientRoot, transactions::Checkpoint, Evaluator,
    Value,
};
use super::{arena, reachable};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};
#[cfg(test)]
mod oracle;
#[cfg(test)]
pub(super) use oracle::with_collection_mode;
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn collect_arm_generation(
        &mut self,
        checkpoint: Checkpoint,
        returned: &[Value],
    ) {
        let frames = self.transaction_frame_checkpoint(checkpoint);
        let identities = self.transaction_argument_cutoff(checkpoint);
        let roots = self.collect_transaction_roots(checkpoint);
        let live = self.collect_generation(frames, identities, &roots, returned);
        self.prune_transaction_new_identities(checkpoint, &live.arguments, &live.identities);
    }
    pub(in super::super) fn collect_final_generation(
        &mut self,
        frames: FrameCheckpoint,
        identities: u64,
        roots: &[TransientRoot],
        returned: &[Value],
    ) {
        let pins = self.pin_value_roots(returned, roots);
        self.collect_generation(frames, identities, roots, returned);
        self.reclaim_dead_state();
        self.release_delta_pins(pins);
    }
    fn collect_generation(
        &mut self,
        frames: FrameCheckpoint,
        identities: u64,
        roots: &[TransientRoot],
        returned: &[Value],
    ) -> reachable::Reachable {
        self.transactions.reclamation.report_reference_metadata();
        let frozen = reachable::Frozen { frames, identities };
        let mut remembered = roots.to_vec();
        remembered.extend_from_slice(&self.transient_roots);
        #[cfg(test)]
        let pin_cutoff = oracle::pin_cutoff(identities);
        #[cfg(not(test))]
        let pin_cutoff = identities;
        remembered.extend(self.pinned_identity_roots(pin_cutoff));
        let mut environments = Vec::new();
        let mut arguments = Vec::new();
        for initial in &self.active_module_initials {
            environments.extend(initial.young_roots(frames));
            arguments.extend(initial.young_arguments(identities));
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(
                "postgres.annotation.generation_root_seeds",
                (remembered.len() + environments.len() + arguments.len() + returned.len()) as u64,
            );
        }
        #[cfg(test)]
        let collect = oracle::collect;
        #[cfg(not(test))]
        let collect = reachable::collect_young;
        let live = collect(
            &self.scopes,
            frozen,
            reachable::RootSeeds {
                environments: &environments,
                arguments: &arguments,
                transient: &remembered,
            },
            returned,
            arena::Arena {
                objects: &self.argument_objects,
                extras: &self.argument_extra_slots,
            },
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        let dead = self
            .scopes
            .allocated_since(frames)
            .filter(|env| !live.environments.contains(env))
            .collect::<Vec<_>>();
        for env in dead {
            self.release_frame(env);
        }
        for id in identities..self.next_builder {
            if !live.arguments.contains(&id) && self.argument_objects.contains_key(&id) {
                self.drop_argument_state(id);
            }
            if !live.identities.contains(&id) && !self.identity_is_pinned(id) {
                self.builder_updates.remove(&id);
            }
        }
        live
    }
    /// Speculative root entrypoints share the module baseline, never private frames.
    pub(in super::super) fn discard_root_allocations(
        &mut self,
        frames: FrameCheckpoint,
        identities: u64,
    ) {
        let checkpoint = self
            .transactions
            .frames
            .iter()
            .position(|frame| frame.scopes == frames && frame.identities == identities)
            .map(Checkpoint)
            .expect("active root checkpoint");
        self.prune_transaction_new_identities(
            checkpoint,
            &FxHashSet::default(),
            &FxHashSet::default(),
        );
        let dead = self.scopes.allocated_since(frames).collect::<Vec<_>>();
        for env in dead {
            self.release_frame(env);
        }
        for id in identities..self.next_builder {
            self.drop_argument_state(id);
        }
    }
}
