use super::*;
use mutation_projection::MutationInfo;

impl ImportCollector {
    pub(super) fn mutation_reference_replaced(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        reads: &[nested::ReceiverRead],
        info: MutationInfo<'_>,
    ) -> bool {
        let Some(proof) = info.proof else {
            return self.reference_path_may_be_replaced(key, path, reads);
        };
        if path.is_empty() || self.fresh_receiver_effects.rebound(key.0, &key.1) {
            return false;
        }
        // Keep the original writer's epoch and shared canonical phase receipts.
        // The collector is local to one file; no history is copied for replay.
        let epoch = std::mem::replace(&mut self.fresh_receiver_effects.epoch, info.metadata.epoch);
        let replaced = {
            let phases = proof.phases.as_ref().map(|phases| phases.borrow());
            self.prior_member_mutations
                .history(key)
                .is_some_and(|(history, complete)| {
                    complete
                        && history.reference_may_be_replaced(
                            path,
                            reads,
                            proof.region,
                            Some(&self.fresh_receiver_effects),
                            phases
                                .as_deref()
                                .filter(|_| proof.owner == Some(info.metadata.context))
                                .map(|phases| {
                                    phases.cursor(
                                        &self.prior_member_mutations,
                                        key,
                                        info.metadata.context,
                                    )
                                }),
                        )
                })
        };
        self.fresh_receiver_effects.epoch = epoch;
        replaced
    }
}
