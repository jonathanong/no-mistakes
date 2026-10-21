use super::*;
use mutation_metadata::MutationMetadata;
use mutation_projection::MutationInfo;
#[path = "mutable_foreign_arguments.rs"]
mod arguments;

// Keep uncertainty in the identity: an ordinary and a recovered possible demand
// for the same source must not promote each other during deferred replay.
type WriteKey = (
    (usize, String),
    ValueMutationPath,
    (CallableId, CallableId),
    Option<String>,
    bool,
    bool,
);
struct ForeignWrite {
    metadata: MutationMetadata,
    completed_at: u32,
}

/// A callable can observe origins from bodies visited later in the same pass.
/// Repeated writes share one demand; no writer retains a copy of root history.
/// Declarations remain conservative summaries because escaped/aliased calls need
/// not appear in the closed direct-call evidence used for exact module captures.
#[derive(Default)]
pub(in super::super::super) struct ForeignReceiverWrites {
    writes: FxHashMap<WriteKey, ForeignWrite>,
    finishing: bool,
}

#[cfg(feature = "test-instrumentation")]
thread_local! {
    pub(in super::super::super) static WORK: std::cell::Cell<[usize; 4]> = const { std::cell::Cell::new([0; 4]) };
}
#[cfg(feature = "test-instrumentation")]
pub(super) fn count(index: usize, amount: usize) {
    WORK.with(|work| {
        let mut values = work.get();
        values[index] += amount;
        work.set(values);
    });
}

impl ImportCollector {
    pub(in super::super) fn retain_foreign_receiver_write(
        &mut self,
        key: &(usize, String),
        path: &[Option<String>],
        read: nested::RootRead,
        info: MutationInfo<'_>,
    ) {
        let (Some(writer), Some(acquired)) = (info.metadata.context, read.1) else {
            return;
        };
        // Module captures keep their saved object. A callable capture can see
        // foreign providers even when a nested closure owns the eventual write.
        if self.foreign_receiver_writes_finishing() {
            return;
        }
        if arguments::forwarded_alias(self, key) {
            return;
        }
        #[cfg(feature = "test-instrumentation")]
        count(0, 1);
        let writes = self
            .foreign_receiver_writes
            .get_or_insert_with(Default::default);
        let write = writes.writes.entry((
            key.clone(),
            path.to_vec(),
            (writer, acquired),
            info.assigned_member.map(str::to_owned),
            info.has_assignment,
            self.receiver_escapes.projecting_possible,
        ));
        write
            .and_modify(|write| {
                write.metadata.foreign_before = write
                    .metadata
                    .foreign_before
                    .min(info.metadata.foreign_before);
                write.metadata.offset = write.metadata.offset.min(info.metadata.offset);
                write.completed_at = write.completed_at.min(info.completed_at);
            })
            .or_insert(ForeignWrite {
                metadata: info.metadata,
                completed_at: info.completed_at,
            });
    }

    /// True while finalization replays ordinary references; module-owned
    /// origins must then be walked directly because no write is retained.
    pub(in super::super) fn foreign_receiver_writes_finishing(&self) -> bool {
        self.foreign_receiver_writes
            .as_ref()
            .is_some_and(|writes| writes.finishing)
    }

    pub(in super::super::super) fn finish_foreign_receiver_writes(&mut self) {
        let Some(mut writes) = self.foreign_receiver_writes.take() else {
            return;
        };
        writes.finishing = true;
        let mut pending: Vec<_> = std::mem::take(&mut writes.writes).into_iter().collect();
        pending.sort_by(|(left, _), (right, _)| left.cmp(right));
        self.foreign_receiver_writes = Some(writes);
        #[cfg(feature = "test-instrumentation")]
        count(1, pending.len());
        let mut providers = fx_map();
        let empty_origins = ReceiverBindingOrigins::default();
        let mut arguments = None;
        let mut aliases = None;
        for ((key, path, (owner, acquired), assigned, has_assignment, possible), mut write) in
            pending
        {
            let (index, possible_index) = providers
                .entry((key.clone(), path.clone()))
                .or_insert_with(|| {
                    let mut possible = Vec::new();
                    let mut members = self
                        .prior_member_mutations
                        .foreign_member_provider_references(&key, &path);
                    if members.iter().any(|((receiver, _, _), _, _)| {
                        self.value_parameter_indexes
                            .get(&receiver.0 .0)
                            .is_some_and(|names| names.contains_key(&receiver.0 .1))
                    }) {
                        possible = arguments
                            .get_or_insert_with(|| {
                                arguments::MemberArguments::new(self.parameter_call_origins())
                            })
                            .expand(&mut members);
                    }
                    let origins = self
                        .receiver_only_bindings
                        .as_deref()
                        .unwrap_or(&empty_origins);
                    let possible = (!possible.is_empty())
                        .then(|| origins.foreign_provider_index(&key, &path, possible));
                    (
                        origins.foreign_provider_index(&key, &path, members),
                        possible,
                    )
                });
            // A direct writer may acquire its own later provider on another call.
            // A nested closure retains its enclosing owner's saved acquisition.
            let references = index.references(
                (owner != acquired).then_some(acquired),
                write.metadata.foreign_before,
            );
            // The declaration order cannot establish an exact replacement or
            // a fresh-object certificate. Preserve the writer, member and scope.
            write.metadata.fresh = false;
            write.metadata.unconditional = false;
            write.metadata.region = None;
            write.metadata.phase = None;
            write.metadata.foreign_before = usize::MAX;
            let info = MutationInfo {
                metadata: write.metadata,
                assigned_member: assigned.as_deref(),
                completed_at: write.completed_at,
                has_assignment,
                proof: None,
            };
            if let Some(possible) = possible_index {
                self.replay_possible_foreign(
                    possible,
                    (owner != acquired).then_some(acquired),
                    info,
                );
            }
            if possible {
                self.replay_possible_foreign(index, (owner != acquired).then_some(acquired), info);
            } else {
                self.replay_foreign(index, references, owner, info, &mut aliases);
            }
        }
        self.foreign_receiver_writes = None;
    }
}

const _: () = assert!(size_of::<Option<Box<ForeignReceiverWrites>>>() == size_of::<usize>());

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutable_foreign_writes_tests.rs"]
mod tests;

#[path = "mutable_foreign_replay.rs"]
mod replay;
