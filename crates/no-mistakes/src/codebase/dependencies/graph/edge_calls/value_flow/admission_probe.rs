use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Test-feature observations belong to one final CallPass request, never a thread.
#[derive(Default)]
pub(super) struct AdmissionProbe {
    mode: admission_mode::Mode,
    without_node: AtomicUsize,
    with_node: AtomicUsize,
    skipped: AtomicUsize,
}
impl AdmissionProbe {
    pub(super) fn original_admission(&self) -> bool {
        self.mode.original()
    }
    pub(super) fn skip_direct_callable(&self) {
        self.skipped.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn admission(&self, node: bool) {
        if node {
            &self.with_node
        } else {
            &self.without_node
        }
        .fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn capture(&self, result: &mut ReplacementProbeSnapshot) {
        result.receiver_initial_admissions_without_target_node =
            self.without_node.load(Ordering::Relaxed);
        result.receiver_initial_admissions_with_target_node =
            self.with_node.load(Ordering::Relaxed);
        result.receiver_original_admission = self.original_admission();
        result.receiver_skipped_direct_callable_admissions = self.skipped.load(Ordering::Relaxed);
    }
}
