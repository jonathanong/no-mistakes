use std::sync::atomic::{AtomicUsize, Ordering};

static ABSENT_FIXTURE_DISPATCHES: AtomicUsize = AtomicUsize::new(0);

pub(super) fn record(root: &std::path::Path) {
    if root.ends_with("forbidden-dependencies-inventory-absent/fixture") {
        ABSENT_FIXTURE_DISPATCHES.fetch_add(1, Ordering::Relaxed);
    }
}

pub(crate) fn absent_fixture_dispatches() -> usize {
    ABSENT_FIXTURE_DISPATCHES.load(Ordering::Relaxed)
}

macro_rules! record_markdown_dispatch {
    ($root:expr) => {
        markdown_dispatch_probe::record($root)
    };
}
