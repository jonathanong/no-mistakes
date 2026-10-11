fn ordinary_call_sites(root: &Path, file: &str, export: &str) -> Value {
    parse_json(
        crate::napi_api::queries::call_sites_json_impl(json!({
            "root": root, "file": file, "exportName": export
        }))
        .unwrap(),
    )
}

fn observed_call_batch(
    root: &Path,
    reports: Value,
) -> (
    Vec<Value>,
    std::sync::Arc<crate::diagnostics::InvocationObserver>,
) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "reports": reports }).to_string(),
        ))
        .unwrap()
    };
    (report_results(&output), observer)
}

fn assert_call_batch_work(
    observer: &crate::diagnostics::InvocationObserver,
    observed_parses: usize,
) {
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["parse.files"], observed_parses as u64, "{work:#?}");
    assert!(
        observer
            .source_read_snapshot()
            .values()
            .all(|reads| *reads == 1),
        "{:?}",
        observer.source_read_snapshot()
    );
}

#[test]
fn prepared_call_sites_preserve_ordinary_catalog_and_runner_config_callers() {
    let source = repo_fixture(&["fixtures", "queries", "call-sites-ordinary-scope"]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let file = "packages/unit/src/subject.ts";
    let standalone = ordinary_call_sites(&root, file, "subject");
    let sites = standalone["callSites"].as_array().unwrap();
    assert_eq!(sites.len(), 2, "{standalone:#?}");
    assert!(sites.iter().all(|site| {
        matches!(
            site["file"].as_str(),
            Some("packages/unit/vitest.config.ts" | "playwright.config.ts")
        )
    }));

    for mixed in [false, true] {
        let mut reports = vec![json!({"type": "callSites", "file": file, "exportName": "subject"})];
        if mixed {
            // Runner discovery prepares the broader catalog independently of edge kinds.
            // Limit edges to calls/imports to avoid unrelated Playwright selector work.
            reports.push(json!({
                "type": "dependents", "files": [file],
                "relationships": ["call", "import"], "tests": ["vitest"]
            }));
        }
        crate::ast::begin_parse_count(&root);
        let (results, observer) = observed_call_batch(&root, json!(reports));
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(results[0], standalone, "mixed={mixed}");
        assert_eq!(counts.len(), 5, "mixed={mixed}: {counts:#?}");
        assert!(counts.values().all(|count| *count == 1), "mixed={mixed}: {counts:#?}");
        // Runner config/helper ASTs use the raw parser gateway before session facts.
        assert_call_batch_work(&observer, if mixed { 3 } else { 5 });
    }
}

#[test]
fn prepared_call_sites_do_not_admit_ignored_sibling_targets() {
    let fixture = crate::test_support::materialize_gitignore_fixture("call-sites-sibling");
    let root = fixture.path().canonicalize().unwrap();
    let standalone = ordinary_call_sites(&root, "target.ts", "used");
    assert_eq!(standalone["callSites"].as_array().unwrap().len(), 1);
    let ignored = ordinary_call_sites(&root, "ignored/sibling.ts", "own");
    let (results, observer) = observed_call_batch(
        &root,
        json!([
            {"type": "callSites", "file": "target.ts", "exportName": "used"},
            {"type": "callSites", "file": "ignored/sibling.ts", "exportName": "own"},
            {"type": "dependencies", "files": ["ignored/sibling.ts"], "relationships": ["call"]}
        ]),
    );
    assert_eq!(results[0], standalone);
    assert_eq!(results[1], ignored);
    assert_call_batch_work(&observer, 3);
}

#[test]
fn prepared_call_sites_keep_recovered_exports_from_malformed_runner_helpers() {
    let source = repo_fixture(&["fixtures", "queries", "call-sites-malformed-runner-helper"]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = ordinary_call_sites(&root, "helper.ts", "used");
    assert_eq!(standalone["callSites"].as_array().unwrap().len(), 2);
    crate::ast::begin_parse_count(&root);
    let (results, observer) = observed_call_batch(
        &root,
        json!([
            {"type": "callSites", "file": "helper.ts", "exportName": "used"},
            {"type": "dependents", "files": ["helper.ts"], "relationships": ["call", "import"], "tests": ["vitest"]}
        ]),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 3, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    assert_eq!(results[0], standalone);
    // The malformed helper is parsed by runner evaluation before session projection.
    assert_call_batch_work(&observer, 2);
}

#[test]
fn prepared_call_sites_reuse_equivalent_canonical_call_graph() {
    let source = repo_fixture(&["fixtures", "queries", "resolved-call-sites"]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = ordinary_call_sites(&root, "target.ts", "used");
    let (results, observer) = observed_call_batch(
        &root,
        json!([
            {"type": "callSites", "file": "target.ts", "exportName": "used"},
            {"type": "callSites", "file": "barrel.ts", "exportName": "forwarded"},
            {"type": "dependencies", "files": ["consumer.ts"], "relationships": ["call"]}
        ]),
    );
    assert_eq!(results[0], standalone);
    assert_eq!(results[1]["callSites"], standalone["callSites"]);
    assert_call_batch_work(&observer, 6);
    let work = observer.snapshot().work;
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert!(work["graph.reuses"] >= 1, "{work:#?}");
}

#[test]
fn prepared_binding_queries_share_graph_and_one_fact_pass() {
    let source = repo_fixture(&["fixtures", "queries", "resolved-call-sites"]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = parse_json(
        crate::napi_api::queries::call_sites_json_impl(json!({
            "root": root, "file": "target.ts", "exportName": "used"
        }))
        .unwrap(),
    );
    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {"type": "callSites", "file": "target.ts", "exportName": "used"},
                {"type": "callSites", "file": "barrel.ts", "exportName": "forwarded"},
                {"type": "dependencies", "files": ["consumer.ts"], "relationships": ["import"]}
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let results = report_results(&output);
    assert_eq!(results[0], standalone);
    assert_eq!(results[1]["callSites"], standalone["callSites"]);
    assert_eq!(counts.len(), 6, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn prepared_module_export_effects_match_standalone() {
    let root = repo_fixture(&["fixtures", "queries", "resolved-effects"]);
    let standalone = parse_json(
        crate::napi_api::effects_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "kind": "resolved", "entry": "entry.ts"
        })))
        .unwrap(),
    );
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root, "reports": [{"type": "effects", "kind": "resolved", "entry": "entry.ts"}]
        })
        .to_string(),
    ))
    .unwrap();
    assert_eq!(report_results(&output)[0], standalone);
}
