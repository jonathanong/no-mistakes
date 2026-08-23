use super::*;
use std::time::Duration;

#[test]
fn timings_mark_and_print() {
    let observer = crate::diagnostics::InvocationObserver::new(false);
    let guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let mut timings = PhaseTimings::start();
    timings.mark("search");
    timings.mark("analysis");

    assert_eq!(timings.phases.len(), 2);
    assert_eq!(timings.phases[0].0, "search");
    assert_eq!(timings.phases[1].0, "analysis");
    assert_eq!(observer.snapshot().timings.len(), 2);
    timings.print_stderr();
    drop(guard);
}

#[test]
fn timings_without_an_observer_print_and_skip_unstarted_marks() {
    let mut skipped = PhaseTimings {
        last: None,
        phases: vec![("search", Duration::from_millis(1))],
    };
    skipped.mark("ignored");
    assert_eq!(skipped.phases.len(), 1);

    let mut timings = PhaseTimings {
        last: Some(std::time::Instant::now()),
        phases: Vec::new(),
    };
    timings.mark("search");
    assert_eq!(timings.phases.len(), 1);
    timings.print_stderr();
}
