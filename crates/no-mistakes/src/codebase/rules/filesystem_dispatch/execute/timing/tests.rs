use super::*;
use std::cell::Cell;

#[test]
fn disabled_observers_skip_label_formatting_and_run_once() {
    let runs = Cell::new(0);
    let value = trace_rule(
        None,
        || panic!("disabled observer formatted a rule timing label"),
        || {
            runs.set(runs.get() + 1);
            42
        },
    );
    assert_eq!(value, 42);
    assert_eq!(runs.get(), 1);
}

#[test]
fn enabled_observer_formats_and_records_rule_label() {
    let observer = InvocationObserver::new(false);
    assert_eq!(
        trace_rule(Some(&observer), || "rules.special".to_owned(), || 42),
        42
    );
    let timings = observer.snapshot().timings;
    assert_eq!(timings.len(), 1);
    assert_eq!(timings[0].label, "rules.special");
    assert_eq!(timings[0].kind, TimingKind::Parallel);
}
