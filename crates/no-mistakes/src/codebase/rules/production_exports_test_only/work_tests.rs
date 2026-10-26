use std::sync::Arc;

#[test]
fn applications_and_barrel_exports_share_inventory_parse_graph_and_incomplete_scan() {
    let source = super::tests::fixture("work-count");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({ "root": root }).to_string(),
        ))
        .unwrap()
    };
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    // Both applications must retain all eight definite findings despite the
    // unrelated computed import; distinct messages keep the applications visible.
    assert_eq!(report["rules"].as_array().unwrap().len(), 16, "{report}");
    assert!(report["advisories"].as_array().unwrap().is_empty());
    let work = observer.snapshot().work;
    assert_eq!(work.get("parse.files"), Some(&4), "{work:#?}");
    assert_eq!(work.get("parse.files_after_extract"), Some(&4), "{work:#?}");
    assert_eq!(work.get("graph.builds"), Some(&1), "{work:#?}");
    assert_eq!(
        work.get("production_exports.incomplete_scans"),
        Some(&1),
        "the universe scan belongs to the request, not each export or application: {work:#?}"
    );
}
