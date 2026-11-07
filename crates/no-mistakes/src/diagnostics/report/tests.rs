use super::*;
use crate::diagnostics::{DiagnosticFilePass, TimingKind};
use std::{path::Path, time::Duration};

#[test]
fn structured_verbose_reports_keep_named_units_and_existing_timing_kinds() {
    let observer = InvocationObserver::new(true);
    observer.record_duration("parallel", Duration::from_millis(3), TimingKind::Parallel);
    observer.increment("source.bytes", 12);
    observer.trace("phase", TimingKind::Serial, || {
        observer.trace_file(
            DiagnosticFilePass::ValueFlowFinalize,
            Path::new("source.ts"),
            12,
            || (),
        );
    });
    let snapshot = observer.verbose_snapshot().unwrap();
    let json = serde_json::to_value(&snapshot).unwrap();
    assert!(json["elapsedMs"].as_f64().unwrap() >= 0.0);
    assert_eq!(json["work"]["source.bytes"], 12);
    assert!(snapshot
        .timings
        .iter()
        .any(|entry| entry.label == "parallel" && entry.duration_ms == 3.0 && entry.parallel));
    assert_eq!(json["hotFiles"][0]["pass"], "value_flow.finalize");
    assert_eq!(json["hotFiles"][0]["path"], "source.ts");
    assert_eq!(json["hotFiles"][0]["units"], 12);
    #[cfg(unix)]
    {
        assert_eq!(json["phases"][0]["label"], "phase");
        for field in [
            "wallMs",
            "userMs",
            "systemMs",
            "utilization",
            "peakRssKiB",
            "peakRssDeltaKiB",
        ] {
            assert!(json["phases"][0][field].is_number(), "{field}: {json}");
        }
        assert!(json["peakRssKiB"].as_u64().unwrap() > 0);
    }
    #[cfg(not(unix))]
    {
        assert!(snapshot.phases.is_empty());
        assert!(json.get("peakRssKiB").is_none());
    }
}

#[test]
fn basic_observers_skip_structured_verbose_collection() {
    let observer = InvocationObserver::new(false);
    assert!(observer.verbose_snapshot().is_none());
    assert!(observer.hot_files.is_none());
    assert!(observer.resources.is_none());
    assert!(observer.hot_file_snapshot().is_empty());
    assert!(observer.resource_snapshot().is_empty());
}
