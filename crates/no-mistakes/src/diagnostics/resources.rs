use super::{InvocationObserver, TimingKind};
use std::fmt::Write;
use std::time::Duration;

mod aggregation;
mod usage;
use super::report::PhaseResourceDiagnostic;
use aggregation::aggregate;
use usage::{sample, Usage};

#[derive(Default)]
pub(super) struct ResourceState {
    active: usize,
    completed: Vec<PhaseResources>,
}

#[derive(Clone)]
struct PhaseResources {
    label: String,
    wall: Duration,
    start: Usage,
    end: Usage,
}

pub(super) struct ResourceGuard<'a> {
    observer: &'a InvocationObserver,
    start: Option<Usage>,
}

impl<'a> ResourceGuard<'a> {
    pub(super) fn start(
        observer: &'a InvocationObserver,
        label: &str,
        kind: TimingKind,
    ) -> Option<Self> {
        let state = observer.resources.as_ref()?;
        // Preparation and dispatch envelopes leave attribution to their sequential children.
        if matches!(label, "invocation.dispatch" | "facts.prepare") {
            return None;
        }
        let top_level = {
            let mut state = state.lock().unwrap();
            let top_level = state.active == 0 && kind == TimingKind::Serial;
            state.active += 1;
            top_level
        };
        let start = top_level.then(sample).flatten();
        Some(Self { observer, start })
    }

    pub(super) fn finish(&self) -> Option<Usage> {
        self.start.and_then(|_| sample())
    }

    pub(super) fn record(&self, label: &str, wall: Duration, end: Option<Usage>) {
        let Some((start, end)) = self.start.zip(end) else {
            return;
        };
        let completed = PhaseResources {
            label: label.to_owned(),
            wall,
            start,
            end,
        };
        self.observer
            .resources
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .completed
            .push(completed);
    }
}

impl Drop for ResourceGuard<'_> {
    fn drop(&mut self) {
        self.observer
            .resources
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .active -= 1;
    }
}

impl InvocationObserver {
    pub(super) fn resource_snapshot(&self) -> Vec<PhaseResourceDiagnostic> {
        self.resource_snapshot_with(aggregate)
    }

    fn resource_snapshot_with(
        &self,
        aggregate: impl FnOnce(Vec<PhaseResources>) -> Vec<PhaseResourceDiagnostic>,
    ) -> Vec<PhaseResourceDiagnostic> {
        let Some(state) = &self.resources else {
            return Vec::new();
        };
        let completed = state.lock().unwrap().completed.clone();
        aggregate(completed)
    }

    pub(super) fn resource_block(&self) -> String {
        format_resources(self.resource_snapshot())
    }
}

fn format_resources(phases: Vec<PhaseResourceDiagnostic>) -> String {
    let mut output = String::new();
    for phase in phases {
        let _ = writeln!(
            output,
            "[cpu] {}: user={:.3}ms sys={:.3}ms util={:.3}x",
            phase.label, phase.user_ms, phase.system_ms, phase.utilization
        );
        let _ = writeln!(
            output,
            "[memory] {}: peak_rss_kib={} delta_kib={}",
            phase.label, phase.peak_rss_kib, phase.peak_rss_delta_kib
        );
    }
    output
}

#[cfg(test)]
mod tests;
