#[test]
fn forbidden_calls_scoped_arguments_preserve_analyze_project_ordinary_call_sites() {
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-scope");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone_check = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root }).to_string(),
        ))
        .unwrap(),
    );
    let findings = standalone_check["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 3, "{standalone_check:#?}");
    assert!(findings.iter().any(|finding| {
        finding["file"] == "src/a/helper.mjs"
            && finding["target"] == "module export `node:child_process#execFile`"
    }));
    assert!(findings.iter().any(|finding| {
        finding["file"] == "src/b/entry.mjs"
            && finding["target"] == "module export `node:child_process#execFile`"
    }));

    let targets = [
        ("src/a/helper.mjs", "execute"),
        ("src/unrelated.mjs", "unrelated"),
    ];
    let standalone_calls = targets
        .iter()
        .map(|(file, export)| {
            let result = ordinary_call_sites(&root, file, export);
            assert_eq!(
                result["callSites"].as_array().unwrap().len(),
                1,
                "{result:#?}"
            );
            result
        })
        .collect::<Vec<_>>();
    let call_reports = targets
        .iter()
        .map(|(file, export)| json!({ "type": "callSites", "file": file, "exportName": export }))
        .collect::<Vec<_>>();
    let (queries_only, observer) = observed_call_batch(&root, json!(call_reports));
    assert_eq!(queries_only, standalone_calls);
    assert!(observer
        .snapshot()
        .work
        .get("facts.call_arguments.collections")
        .is_none());

    crate::ast::begin_parse_count(&root);
    // The second query asks about a sibling outside every argument application's
    // filters. Adding the check must preserve its ordinary call-site projection.
    let mut mixed = vec![json!({ "type": "check" })];
    mixed.extend(call_reports);
    let (results, observer) = observed_call_batch(&root, json!(mixed));
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(results[0], standalone_check);
    assert_eq!(&results[1..], standalone_calls.as_slice());
    assert_each_indexable_file_parsed_once(&root, &counts);
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("facts.call_arguments.collections"),
        Some(&4)
    );
}
