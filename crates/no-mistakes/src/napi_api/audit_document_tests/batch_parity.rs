use super::*;

#[test]
fn mixed_audit_batch_preserves_reverse_query_diagnostics_and_prepared_work() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/queries-incomplete/fixture");
    let reports = vec![
        json!({"type":"callSites", "file":"target.ts", "exportName":"unknown"}),
        json!({"type":"callSites", "file":"target.ts", "exportName":"used"}),
    ];
    let run = |reports| {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let result = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::napi_api::analyze_project::analyze_project_json_impl(
                json!({"root":root,"reports":reports}),
            )
            .unwrap()
        });
        (
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            observer,
        )
    };
    let (baseline, baseline_work) = run(reports.clone());
    assert_eq!(baseline["reports"][0]["result"]["partial"], true);
    assert_eq!(
        baseline["reports"][0]["result"]["incompleteFiles"][0]["file"],
        "broken.ts"
    );
    assert_eq!(
        baseline["reports"][0]["result"]["incompleteFiles"][0]["reason"],
        "parse-error"
    );
    assert_eq!(baseline_work.snapshot().work["query_diagnostics.builds"], 1);

    let mut mixed_reports = reports;
    let mut audit = request();
    audit["type"] = json!("testsAudit");
    // An artifact first in the batch must not own or broaden the repository scope.
    mixed_reports.insert(0, audit);
    let (mixed, mixed_work) = run(mixed_reports);
    assert_eq!(mixed["reports"][1], baseline["reports"][0]);
    assert_eq!(mixed["reports"][2], baseline["reports"][1]);
    assert_eq!(mixed_work.snapshot().work, baseline_work.snapshot().work);
    assert_eq!(
        mixed_work.source_read_snapshot(),
        baseline_work.source_read_snapshot()
    );
}
