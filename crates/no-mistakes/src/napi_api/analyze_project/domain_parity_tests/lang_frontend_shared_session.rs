// Exercise the public prepared-analysis boundary. A private edge helper can
// accidentally hide different graph/queue/route file universes.
fn public_language_reports(root: &Path, reports: Value) -> (Vec<Value>, u64) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = crate::diagnostics::with_observer(Some(std::sync::Arc::clone(&observer)), || {
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "reports": reports }).to_string(),
        ))
    })
    .unwrap();
    let count = observer
        .snapshot()
        .work
        .get("lang_frontends.file_extractions")
        .copied()
        .unwrap_or(0);
    (report_results(&output), count)
}

#[test]
fn public_queue_routes_and_graph_share_language_extraction_per_fresh_request() {
    let root = repo_fixture(&["fixtures", "lang-frontends", "python-celery-django"]);
    let requests = [
        json!({ "type": "queues" }),
        json!({ "type": "serverRoutes" }),
        json!({ "type": "dependencies", "files": ["app/enqueue.py"] }),
    ];
    let standalone = requests
        .iter()
        .map(|request| public_language_reports(&root, json!([request])))
        .collect::<Vec<_>>();
    let expected = standalone[0].1;
    assert!(expected > 1, "saved fixture must exercise real extraction");
    assert!(standalone.iter().all(|(_, count)| *count == expected));
    assert!(!standalone[0].0[0]["producers"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!standalone[1].0[0]["routes"].as_array().unwrap().is_empty());
    assert!(!standalone[2].0[0]["files"].as_array().unwrap().is_empty());
    // A repeated public call must create fresh request state, while each call's
    // three real consumers share the extraction and retain their whole reports.
    for _ in 0..2 {
        let (combined, count) = public_language_reports(&root, json!(requests));
        assert_eq!(combined.len(), requests.len());
        for (index, report) in combined.iter().enumerate() {
            assert_eq!(report, &standalone[index].0[0]);
        }
        assert_eq!(
            count, expected,
            "one extraction across actual public consumers"
        );
    }
}
