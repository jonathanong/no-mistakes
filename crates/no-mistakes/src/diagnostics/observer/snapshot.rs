use super::*;

impl InvocationObserver {
    /// Timeout snapshots must retain available work even if a timing writer
    /// owns the mutex. Absence is reported explicitly, never as a zero duration.
    pub(in crate::diagnostics) fn partial_snapshot(&self) -> (DiagnosticsSnapshot, bool) {
        let (recorded, available) = match self.timings.try_lock() {
            Ok(timings) => (timings.clone(), true),
            Err(_) => (Vec::new(), false),
        };
        (self.snapshot_from(recorded), available)
    }

    pub(super) fn snapshot_from(&self, recorded: Vec<TimingDiagnostic>) -> DiagnosticsSnapshot {
        let mut grouped = BTreeMap::<(String, TimingKind), Duration>::new();
        for entry in recorded {
            *grouped.entry((entry.label, entry.kind)).or_default() += entry.duration;
        }
        let mut timings = grouped
            .into_iter()
            .map(|((label, kind), duration)| TimingDiagnostic {
                label,
                duration,
                kind,
            })
            .collect::<Vec<_>>();
        timings.sort_by(|a, b| {
            let rank = timing_order(&a.label).cmp(&timing_order(&b.label));
            if rank != std::cmp::Ordering::Equal {
                return rank;
            }
            let is_rule = |label: &str| {
                label.starts_with("analysis.rules.")
                    || label.starts_with("analysis.filesystem_rules.")
            };
            match (is_rule(&a.label), is_rule(&b.label)) {
                (true, true) => b
                    .duration
                    .cmp(&a.duration)
                    .then_with(|| a.label.cmp(&b.label)),
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                (false, false) => (a.label.as_str(), a.kind).cmp(&(b.label.as_str(), b.kind)),
            }
        });
        DiagnosticsSnapshot {
            timings,
            work: self
                .work
                .as_ref()
                .map(|work| work.snapshot())
                .unwrap_or_default(),
        }
    }
}
