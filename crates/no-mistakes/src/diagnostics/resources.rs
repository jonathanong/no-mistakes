use super::{InvocationObserver, TimingKind};
use std::fmt::Write;
use std::time::Duration;

mod usage;
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
        // The dispatch envelope is transparent so its sequential children own attribution.
        if label == "invocation.dispatch" {
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
    pub(super) fn resource_block(&self) -> String {
        self.resource_block_with(format_resources)
    }

    fn resource_block_with(&self, format: impl FnOnce(Vec<PhaseResources>) -> String) -> String {
        let Some(state) = &self.resources else {
            return String::new();
        };
        let completed = state.lock().unwrap().completed.clone();
        format(completed)
    }
}

fn format_resources(completed: Vec<PhaseResources>) -> String {
    let mut output = String::new();
    for phase in completed {
        let user = phase.end.user.saturating_sub(phase.start.user);
        let sys = phase.end.sys.saturating_sub(phase.start.sys);
        let util = if phase.wall.is_zero() {
            0.0
        } else {
            (user + sys).as_secs_f64() / phase.wall.as_secs_f64()
        };
        let _ = writeln!(
            output,
            "[cpu] {}: user={:.3}ms sys={:.3}ms util={:.3}x",
            phase.label,
            user.as_secs_f64() * 1000.0,
            sys.as_secs_f64() * 1000.0,
            util
        );
        let _ = writeln!(
            output,
            "[memory] {}: peak_rss_kib={} delta_kib={}",
            phase.label,
            phase.end.peak_rss_kib,
            phase
                .end
                .peak_rss_kib
                .saturating_sub(phase.start.peak_rss_kib)
        );
    }
    output
}

#[cfg(test)]
mod tests;
