use super::run_all;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn prepare_join_serial_time_does_not_exceed_its_wall_time() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::with_progress(true, true);
    let results = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root, None, None).unwrap()
    };
    assert!(results
        .rules
        .iter()
        .any(|finding| { finding.rule == no_mistakes::codebase::rules::FINITE_SET_CONSISTENCY }));

    let snapshot = observer.snapshot();
    let wall = snapshot
        .timings
        .iter()
        .find(|timing| {
            timing.label == "check.prepare"
                && timing.kind == no_mistakes::diagnostics::TimingKind::Serial
        })
        .expect("check.prepare records the join wall")
        .duration;
    let serial = snapshot
        .timings
        .iter()
        .filter(|timing| {
            timing.label.starts_with("check.prepare")
                && timing.kind == no_mistakes::diagnostics::TimingKind::Serial
        })
        .map(|timing| timing.duration)
        .sum::<Duration>();
    assert!(
        serial <= wall,
        "prepare serial {serial:?} exceeded wall {wall:?}: {:#?}",
        snapshot.timings
    );
    for label in ["check.prepare.catalog", "check.prepare.independent"] {
        let timing = snapshot
            .timings
            .iter()
            .find(|timing| timing.label == label)
            .unwrap_or_else(|| panic!("missing {label} in {:#?}", snapshot.timings));
        assert_eq!(
            timing.kind,
            no_mistakes::diagnostics::TimingKind::Parallel,
            "{label} must not add into the prepare wall"
        );
        assert!(
            timing.duration <= wall,
            "{label} {:?} exceeded wall {wall:?}",
            timing.duration
        );
    }
    assert_eq!(snapshot.work.get("check.prepare.catalog"), Some(&1));
    assert_eq!(snapshot.work.get("check.prepare.independent"), Some(&1));
}
