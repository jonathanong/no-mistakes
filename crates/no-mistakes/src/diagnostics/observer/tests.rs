use super::*;

#[test]
fn verbose_timings_block_matches_historical_bytes() {
    // Literal bytes of the ordinary `--verbose-timings` block before timeout
    // snapshots grew a separate formatter. Total and RSS are injected so the
    // comparison does not depend on the live clock or `getrusage`.
    let observer = InvocationObserver::new(true);
    observer.record_duration("prepare", Duration::from_millis(3), TimingKind::Serial);
    observer.record_duration(
        "analysis.rules",
        Duration::from_millis(5),
        TimingKind::Parallel,
    );
    observer.increment("graph.builds", 1);
    observer.increment("source.reads", 7);
    let text = observer.verbose_timings_block(Duration::from_millis(12), Some(4096));
    assert_eq!(
        text,
        "\
[timing] prepare: 3.000ms
[timing] analysis.rules: 5.000ms (parallel; non-additive)
[timing] total: 12.000ms
[work] graph.builds: 1
[work] source.reads: 7
[memory] peak_rss_kib: 4096
"
    );
}
