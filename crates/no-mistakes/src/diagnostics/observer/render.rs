use super::super::memory::memory_line;
use super::TimingDiagnostic;
use super::{DiagnosticsSnapshot, TimingKind};
use std::fmt::Write;
use std::time::Duration;

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

impl super::InvocationObserver {
    /// Ordinary `--verbose-timings` bytes for a fixed total and RSS sample.
    /// Live elapsed time and `getrusage` stay outside this helper so tests can
    /// lock the historical block exactly.
    pub(in crate::diagnostics) fn verbose_timings_block(
        &self,
        total: Duration,
        peak_rss_kib: Option<u64>,
    ) -> String {
        let mut snapshot = self.snapshot();
        snapshot.timings.push(TimingDiagnostic {
            label: "total".to_string(),
            duration: total,
            kind: TimingKind::Serial,
        });
        let mut output = format_block(&snapshot, self.verbose, peak_rss_kib);
        output.push_str(&self.resource_block());
        output.push_str(&self.hot_file_block());
        output
    }
}
