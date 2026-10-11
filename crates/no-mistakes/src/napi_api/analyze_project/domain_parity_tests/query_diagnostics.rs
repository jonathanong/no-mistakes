#[test]
fn partial_call_sites_keep_malformed_sibling_diagnostics_in_their_own_projection() {
    let fixture = crate::test_support::materialize_gitignore_fixture("partial-call-sites-batch");
    let root = fixture.path().canonicalize().unwrap();
    let standalone = ordinary_call_sites(&root, "target.ts", "unknown");
    assert!(standalone["callSites"].as_array().unwrap().is_empty());
    assert_eq!(standalone["partial"], true);
    assert_eq!(standalone["incompleteFiles"].as_array().unwrap().len(), 2);
    let helper = ordinary_call_sites(&root, "ignored/helper.ts", "helper");
    assert_eq!(helper["incompleteFiles"].as_array().unwrap().len(), 3);

    crate::ast::begin_parse_count(&root);
    let (results, observer) = observed_call_batch(
        &root,
        json!([
            {"type": "callSites", "file": "target.ts", "exportName": "unknown"},
            {"type": "callSites", "file": "ignored/helper.ts", "exportName": "helper"},
            {"type": "dependencies", "files": ["ignored/helper.ts"], "relationships": ["call"]}
        ]),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(results[0], standalone);
    assert_eq!(results[1], helper);
    assert_eq!(counts.len(), 5, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    assert_call_batch_work(&observer, 5);
    assert_eq!(observer.snapshot().work.get("symbol_index.builds"), None);
}

#[test]
fn partial_call_sites_keep_runner_helper_diagnostics_when_mixed_with_runner_reports() {
    let source = repo_fixture(&["fixtures", "queries", "partial-call-sites-ordinary-scope"]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let file = "packages/unit/src/subject.ts";
    let standalone = ordinary_call_sites(&root, file, "subject");
    assert_eq!(standalone["partial"], true);
    assert_eq!(standalone["callSites"].as_array().unwrap().len(), 2);
    assert_eq!(standalone["incompleteFiles"][0]["file"], "packages/unit/setup.ts");
    assert_eq!(standalone["incompleteFiles"][0]["reason"], "parse-error");

    for mixed in [false, true] {
        let mut reports = vec![json!({"type": "callSites", "file": file, "exportName": "subject"})];
        if mixed {
            // The runner catalog parses setup before the shared union fact pass.
            reports.push(json!({
                "type": "dependents", "files": [file],
                "relationships": ["call", "import"], "tests": ["vitest"]
            }));
        }
        crate::ast::begin_parse_count(&root);
        let (results, observer) = observed_call_batch(&root, json!(reports));
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(results[0], standalone, "mixed={mixed}");
        assert_eq!(counts.len(), 5, "{counts:#?}");
        assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
        // Runner evaluation parses configs and the recovered setup program
        // before session fact projection; the physical counts above include both.
        assert_call_batch_work(&observer, if mixed { 2 } else { 5 });
    }
}
