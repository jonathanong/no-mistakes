use super::ensure_registered;
use crate::diagnostics::{with_observer, InvocationObserver};
use no_mistakes_ts_facts::hooks::{record_work, trace_current_file, trace_file, FilePass};
use std::path::Path;

#[test]
fn ts_facts_hooks_handle_disabled_and_active_observers() {
    ensure_registered();
    with_observer(None, || {
        record_work("facts.call_arguments.collections", 1);
        assert_eq!(trace_file(FilePass::Parse, Path::new("a.ts"), 4, || 2), 2);
        assert_eq!(trace_current_file(FilePass::ValueFlowExtract, 4, || 3), 3);
    });
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        record_work("facts.call_arguments.collections", 2);
        assert_eq!(trace_file(FilePass::Parse, Path::new("b.ts"), 8, || 5), 5);
        assert_eq!(trace_current_file(FilePass::ValueFlowFinalize, 8, || 7), 7);
    });
    assert_eq!(
        observer.snapshot().work["facts.call_arguments.collections"],
        2
    );
}
