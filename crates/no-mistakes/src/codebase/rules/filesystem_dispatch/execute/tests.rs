use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::diagnostics::{InvocationObserver, TimingKind};
use std::sync::Arc;

#[test]
fn trace_rule_records_only_verbose_observers() {
    let verbose = InvocationObserver::new(true);
    let sources = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&[])),
        Some(verbose.clone()),
    );
    assert_eq!(trace_rule(&sources, "special", || 42), 42);
    let timings = verbose.snapshot().timings;
    assert_eq!(timings.len(), 1);
    assert_eq!(timings[0].label, "filesystem_rule.special");
    assert_eq!(timings[0].kind, TimingKind::Parallel);

    for observer in [None, Some(InvocationObserver::new(false))] {
        let sources =
            SourceStore::new_observed(Arc::new(FileInventory::from_paths(&[])), observer.clone());
        assert_eq!(trace_rule(&sources, "untimed", || 7), 7);
        assert!(observer.is_none_or(|observer| observer.snapshot().timings.is_empty()));
    }
}
