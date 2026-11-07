use super::InvocationObserver;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticTiming {
    pub label: String,
    pub duration_ms: f64,
    pub parallel: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseResourceDiagnostic {
    pub label: String,
    pub wall_ms: f64,
    pub user_ms: f64,
    pub system_ms: f64,
    pub utilization: f64,
    #[serde(rename = "peakRssKiB")]
    pub peak_rss_kib: u64,
    #[serde(rename = "peakRssDeltaKiB")]
    pub peak_rss_delta_kib: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotFileDiagnostic {
    pub pass: super::DiagnosticFilePass,
    pub path: String,
    pub duration_ms: f64,
    pub units: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerboseDiagnostics {
    pub elapsed_ms: f64,
    pub timings: Vec<DiagnosticTiming>,
    pub work: BTreeMap<&'static str, u64>,
    pub phases: Vec<PhaseResourceDiagnostic>,
    pub hot_files: Vec<HotFileDiagnostic>,
    #[serde(rename = "peakRssKiB", skip_serializing_if = "Option::is_none")]
    pub peak_rss_kib: Option<u64>,
}

impl InvocationObserver {
    pub fn verbose_snapshot(&self) -> Option<VerboseDiagnostics> {
        if !self.verbose() {
            return None;
        }
        let snapshot = self.snapshot();
        Some(VerboseDiagnostics {
            elapsed_ms: self.snapshot_clock().1.as_secs_f64() * 1000.0,
            timings: snapshot
                .timings
                .into_iter()
                .map(|timing| DiagnosticTiming {
                    label: timing.label,
                    duration_ms: timing.duration.as_secs_f64() * 1000.0,
                    parallel: timing.kind == super::TimingKind::Parallel,
                })
                .collect(),
            work: snapshot.work,
            phases: self.resource_snapshot(),
            hot_files: self.hot_file_snapshot(),
            peak_rss_kib: super::memory::peak_rss_kib(),
        })
    }
}

#[cfg(test)]
mod tests;
