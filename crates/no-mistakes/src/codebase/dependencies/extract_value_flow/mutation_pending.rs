use super::*;

pub(super) fn append<R: nested::RootCoordinate>(
    pending: &mut super::queue::Queue<PendingMutation<R>>,
    exit_position: usize,
    key: (usize, String),
    root_read: R,
    references: reference_output::ReferenceOutput<nested::ReceiverWrite<R>>,
) {
    #[cfg(all(test, feature = "test-instrumentation"))]
    super::queue::work_counts::record(4);
    pending.append_with_exit(
        exit_position,
        references.into_parts(),
        |reference| (reference, false, true, true),
        (
            ((key, Vec::new()), Vec::new(), root_read),
            true,
            false,
            false,
        ),
    );
    #[cfg(feature = "test-instrumentation")]
    record_queue(pending.len(), 0);
}

#[cfg(feature = "test-instrumentation")]
thread_local! {
    static QUEUE_COUNTS: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
}

#[cfg(feature = "test-instrumentation")]
pub(super) fn record_queue(len: usize, allocations: usize) {
    QUEUE_COUNTS.with(|cell| {
        let mut counts = cell.get();
        counts[0] += allocations;
        counts[1] = counts[1].max(len);
        counts[2] += 1;
        cell.set(counts);
    });
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "mutation_pending_probe_tests.rs"]
mod probe_tests;
