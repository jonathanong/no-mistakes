use std::time::{Duration, Instant};

pub struct PhaseTimings {
    last: Option<Instant>,
    pub(crate) phases: Vec<(&'static str, Duration)>,
}

impl PhaseTimings {
    pub fn start() -> Self {
        Self {
            last: crate::diagnostics::current().map(|_| Instant::now()),
            phases: Vec::new(),
        }
    }

    pub fn mark(&mut self, label: &'static str) {
        let Some(last) = self.last else {
            return;
        };
        let now = Instant::now();
        let duration = now.duration_since(last);
        self.phases.push((label, duration));
        self.last = Some(now);
        if let Some(observer) = crate::diagnostics::current() {
            observer.record_duration(label, duration, crate::diagnostics::TimingKind::Serial);
        }
    }

    pub fn print_stderr(&self) {
        if crate::diagnostics::current().is_some() {
            return;
        }
        for (label, duration) in &self.phases {
            eprintln!("[timing] {label}: {:.3}ms", duration.as_secs_f64() * 1000.0);
        }
    }
}

#[cfg(test)]
mod tests;
