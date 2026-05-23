use super::*;

#[test]
fn timings_mark_and_print() {
    let mut timings = PhaseTimings::start();
    timings.mark("search");
    timings.mark("analysis");

    assert_eq!(timings.phases.len(), 2);
    assert_eq!(timings.phases[0].0, "search");
    assert_eq!(timings.phases[1].0, "analysis");
    timings.print_stderr();
}
