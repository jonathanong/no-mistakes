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
    budgeting: bool,
    pub(in super::super) work: WorkBudget,
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
    pub(in super::super) fn begin_receiver_finalization(&mut self) {
        if self.collect_value_flow || self.collect_per_item_calls {
            self.foreign_receiver_writes
                .get_or_insert_with(Default::default)
                .budgeting = true;
        }
    }

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
}

const _: () = assert!(size_of::<Option<Box<ForeignReceiverWrites>>>() == size_of::<usize>());

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutable_foreign_writes_tests.rs"]
mod tests;

#[path = "mutable_foreign_replay.rs"]
mod replay;

#[path = "foreign_finish.rs"]
mod finish;
