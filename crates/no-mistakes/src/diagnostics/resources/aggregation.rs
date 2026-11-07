use super::{PhaseResourceDiagnostic, PhaseResources};
use crate::fx::fx_map;
use std::time::Duration;

#[derive(Default)]
struct Totals {
    wall: Duration,
    user: Duration,
    system: Duration,
    peak_rss_kib: u64,
    peak_rss_delta_kib: u64,
}

pub(super) fn aggregate(completed: Vec<PhaseResources>) -> Vec<PhaseResourceDiagnostic> {
    let mut by_label = fx_map();
    let mut totals = Vec::<(String, Totals)>::new();
    for phase in completed {
        let index = *by_label.entry(phase.label.clone()).or_insert_with(|| {
            totals.push((phase.label, Totals::default()));
            totals.len() - 1
        });
        let total = &mut totals[index].1;
        total.wall += phase.wall;
        total.user += phase.end.user.saturating_sub(phase.start.user);
        total.system += phase.end.sys.saturating_sub(phase.start.sys);
        total.peak_rss_kib = total.peak_rss_kib.max(phase.end.peak_rss_kib);
        total.peak_rss_delta_kib += phase
            .end
            .peak_rss_kib
            .saturating_sub(phase.start.peak_rss_kib);
    }
    totals
        .into_iter()
        .map(|(label, total)| PhaseResourceDiagnostic {
            label,
            wall_ms: total.wall.as_secs_f64() * 1000.0,
            user_ms: total.user.as_secs_f64() * 1000.0,
            system_ms: total.system.as_secs_f64() * 1000.0,
            utilization: if total.wall.is_zero() {
                0.0
            } else {
                (total.user + total.system).as_secs_f64() / total.wall.as_secs_f64()
            },
            peak_rss_kib: total.peak_rss_kib,
            peak_rss_delta_kib: total.peak_rss_delta_kib,
        })
        .collect()
}
