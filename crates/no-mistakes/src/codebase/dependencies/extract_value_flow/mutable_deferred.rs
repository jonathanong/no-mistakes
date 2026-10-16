use super::*;
use mutation_metadata::MutationMetadata;
use mutation_projection::MutationInfo;

#[derive(Default)]
pub(in super::super) struct DeferredMutations {
    pending: Vec<DeferredMutation>,
    indexes: FxHashMap<(u32, Option<CallableId>, u64), usize>,
}
struct DeferredMutation {
    sources: Vec<(
        nested::ReceiverWrite,
        Option<mutable_origins::RootVersion>,
        bool,
    )>,
    metadata: MutationMetadata,
    assigned_member: Option<String>,
    completed_at: u32,
    has_assignment: bool,
    proof: DeferredProof,
}
pub(super) struct DeferredProof {
    pub(super) phases:
        Option<std::rc::Rc<std::cell::RefCell<active_reference_phases::ActiveReferencePhases>>>,
    pub(super) region: Option<u32>,
    pub(super) owner: Option<Option<CallableId>>,
}
impl DeferredMutations {
    fn push(&mut self, mut mutation: DeferredMutation) {
        // One normal AST visit supplies one physical writer's metadata. Group
        // its alias paths so replay retains one shared traversal, not one per path.
        let key = (
            mutation.metadata.offset,
            mutation.metadata.context,
            mutation.metadata.epoch,
        );
        if let Some(index) = self.indexes.get(&key) {
            self.pending[*index].sources.append(&mut mutation.sources);
        } else {
            self.indexes.insert(key, self.pending.len());
            self.pending.push(mutation);
        }
    }
}
impl ImportCollector {
    pub(in super::super) fn begin_deferred_mutable_writes(
        &mut self,
    ) -> Option<Box<DeferredMutations>> {
        self.deferred_mutable_writes.replace(Box::default())
    }
    pub(super) fn defer_mutable_projection(
        &mut self,
        key: &(usize, String),
        coordinates: (&[Option<String>], &[nested::ReceiverRead], nested::RootRead),
        info: MutationInfo<'_>,
        blocked: bool,
        version: mutable_origins::RootVersion,
    ) -> bool {
        if self.deferred_mutable_writes.is_none() {
            return false;
        }
        let proof = DeferredProof {
            phases: self.fresh_receiver_effects.phases.clone(),
            region: self
                .receiver_conditional_region
                .map(|(token, _)| token.get()),
            owner: self.destructuring_phase_owner,
        };
        let frame = self
            .deferred_mutable_writes
            .as_mut()
            .expect("checked deferred frame");
        frame.push(DeferredMutation {
            sources: vec![(
                (
                    (key.clone(), coordinates.0.to_vec()),
                    coordinates.1.to_vec(),
                    coordinates.2,
                ),
                Some(version),
                blocked,
            )],
            metadata: info.metadata,
            assigned_member: info.assigned_member.map(str::to_string),
            completed_at: info.completed_at,
            has_assignment: info.has_assignment,
            proof,
        });
        true
    }
    pub(in super::super) fn finish_deferred_mutable_writes(
        &mut self,
        previous: Option<Box<DeferredMutations>>,
    ) {
        let frame = self.deferred_mutable_writes.take();
        self.deferred_mutable_writes = previous;
        let Some(frame) = frame else { return };
        if let Some(parent) = self.deferred_mutable_writes.as_mut() {
            for mutation in frame.pending {
                parent.push(mutation);
            }
            return;
        }
        for deferred in frame.pending {
            let mut references = Vec::new();
            let info = MutationInfo {
                metadata: deferred.metadata,
                assigned_member: deferred.assigned_member.as_deref(),
                completed_at: deferred.completed_at,
                has_assignment: deferred.has_assignment,
                proof: Some(&deferred.proof),
            };
            for (((key, path), reads, root_read), version, blocked) in deferred.sources {
                let current = self.mutable_receiver_version(&key, root_read);
                let same = version
                    .zip(current)
                    .is_some_and(|(before, after)| before.same_selected(&after));
                if blocked && same {
                    continue;
                }
                if let Some(current) = current.filter(|_| {
                    !same || !self.mutation_reference_replaced(&key, &path, &reads, info)
                }) {
                    self.append_mutable_receiver_references_unchecked(
                        &key,
                        (&path, &reads, root_read),
                        &mut references,
                        deferred.metadata.foreign_before,
                        current,
                    );
                }
            }
            let pending = references
                .into_iter()
                .map(|reference| (reference, false, true, true))
                .collect();
            let mut affected = mutation_projection::Affected::default();
            self.project_member_mutation::<nested::RootRead>(
                mutation_projection::PendingQueue::from_vec(pending),
                None,
                info,
                &mut affected,
            );
            self.invalidate_mutation_bindings(
                &affected,
                info.metadata.context,
                info.metadata.offset,
                true,
            );
        }
    }
}
