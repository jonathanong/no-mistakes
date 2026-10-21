use super::super::memory::memory_line;
use super::{DiagnosticsSnapshot, TimingKind};
use std::fmt::Write;

/// Bytes for one ordinary diagnostics block. `render_stderr` prints this
/// string unchanged; timeout snapshots use a separate `PARTIAL` formatter.
pub(super) fn format_block(
    snapshot: &DiagnosticsSnapshot,
    verbose: bool,
    peak_rss_kib: Option<u64>,
) -> String {
    let mut output = String::new();
    for entry in &snapshot.timings {
        let suffix = match entry.kind {
            TimingKind::Serial => "",
            TimingKind::Parallel => " (parallel; non-additive)",
        };
        let _ = writeln!(
            output,
            "[timing] {}: {:.3}ms{}",
            entry.label,
            entry.duration.as_secs_f64() * 1000.0,
            suffix
        );
    }
    if verbose {
        for (metric, count) in &snapshot.work {
            let _ = writeln!(output, "[work] {metric}: {count}");
        }
        if let Some(line) = memory_line(verbose, peak_rss_kib) {
            output.push_str(&line);
        }
    }
    output
}
