use super::*;

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
