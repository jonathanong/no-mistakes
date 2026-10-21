thread_local! {
    pub(super) static SCANS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn record(bindings: usize) {
    SCANS.with(|count| count.set(count.get() + bindings));
}

macro_rules! record_binding_scan {
    ($bindings:expr) => {
        binding_scan_count::record($bindings);
    };
}
