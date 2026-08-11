use super::*;

#[test]
fn timing_metadata_omits_unknown_check_runner_labels() {
    assert_eq!(
        timing_metadata("queues"),
        Some((
            "analysis.queues",
            no_mistakes::diagnostics::TimingKind::Parallel,
        ))
    );
    assert_eq!(timing_metadata("not-a-check-timing"), None);
}
