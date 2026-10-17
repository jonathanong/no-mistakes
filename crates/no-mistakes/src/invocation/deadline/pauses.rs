use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub(in crate::invocation::deadline) static PAUSE_WHILE_ODD: AtomicBool = AtomicBool::new(false);
pub(in crate::invocation::deadline) static PAUSE_ODD_REACHED: AtomicBool = AtomicBool::new(false);
pub(in crate::invocation::deadline) static ODD_SPINS: AtomicU64 = AtomicU64::new(0);
pub(in crate::invocation::deadline) static PAUSE_BEFORE_SECOND: AtomicBool = AtomicBool::new(false);
pub(in crate::invocation::deadline) static PAUSE_SECOND_REACHED: AtomicBool =
    AtomicBool::new(false);

pub(super) fn after_odd_publish() {
    if PAUSE_WHILE_ODD.load(Ordering::Acquire) {
        PAUSE_ODD_REACHED.store(true, Ordering::Release);
        while PAUSE_WHILE_ODD.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
}

pub(super) fn on_odd_generation() {
    ODD_SPINS.fetch_add(1, Ordering::Relaxed);
}

pub(super) fn before_second_load() {
    if PAUSE_BEFORE_SECOND.load(Ordering::Acquire) {
        PAUSE_SECOND_REACHED.store(true, Ordering::Release);
        while PAUSE_BEFORE_SECOND.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
}

impl super::ActiveDeadline {
    // Tests poison the process-wide lock on purpose. Production never calls this.
    #[allow(
        dead_code,
        reason = "only the deadline tests clear a poisoned write lock"
    )]
    pub(in crate::invocation::deadline) fn clear_poison_for_test(&self) {
        self.lock.clear_poison();
    }
}
