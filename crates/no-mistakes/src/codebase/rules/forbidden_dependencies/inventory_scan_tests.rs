use std::sync::atomic::{AtomicUsize, Ordering};

static ABSENT_FIXTURE_SCANS: AtomicUsize = AtomicUsize::new(0);

pub(super) fn record(root: &std::path::Path) {
    if root.ends_with("forbidden-dependencies-inventory-absent/fixture") {
        ABSENT_FIXTURE_SCANS.fetch_add(1, Ordering::Relaxed);
    }
}

pub(super) fn absent_fixture_scans() -> usize {
    ABSENT_FIXTURE_SCANS.load(Ordering::Relaxed)
}

macro_rules! record_inventory_scan {
    ($root:expr) => {
        super::inventory_scan_tests::record($root)
    };
}
