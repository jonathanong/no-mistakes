//! Reuse a final compaction only across a proven graph/root identity boundary.
use super::{
    super::{Environment, Evaluator, Value},
    frames,
};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

type Live = (FxHashSet<u64>, FxHashSet<u64>);
#[cfg(not(test))]
#[path = "final_prune_runtime.rs"]
mod hooks;
#[cfg(test)]
#[path = "final_prune_tests.rs"]
mod hooks;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn can_reuse_final_live(
        &self,
        preserved: Environment,
        original_captures_empty: bool,
        final_invalidated_empty: bool,
        indices: &FxHashMap<Environment, Environment>,
    ) -> bool {
        if hooks::original_prune() {
            return false;
        }
        // Final prefix/module/extra restoration reproduces the last compaction.
        // Empty invalidation makes both taint and disconnect writes no-ops;
        // empty captures make capture restoration/refresh no-ops.
        original_captures_empty
            && final_invalidated_empty
            && self.captured_bindings.is_empty()
            && self.active_module_initials.iter().all(|initial| initial.arguments().next().is_none())
            && self.transient_roots.iter().all(|root| {
                !matches!(root, super::super::temporaries::TransientRoot::Environment(env) if *env >= preserved)
            })
            && indices.iter().all(|(old, new)| old == new)
    }

    pub(super) fn prune_final_alternative_state(&mut self, returned: &[Value], live: Option<Live>) {
        let Some((arguments, identities)) = live else {
            self.prune_alternative_state(returned);
            return;
        };
        hooks::record_reuse();
        // All retained frames now seed the same closure. No snapshot-only
        // argument roots or moved transient environments may be substituted.
        // The arena was already pruned; only finalized mutation masks changed.
        frames::prune_mutations(
            frames::MutationState {
                deleted: &mut self.deleted_argument_slots,
                definite: &mut self.definite_deleted_argument_slots,
                disconnected: &mut self.disconnected_argument_slots,
                invalidated: &mut self.invalidated_builders,
                updates: &mut self.builder_updates,
            },
            &arguments,
            &identities,
        );
    }
}
