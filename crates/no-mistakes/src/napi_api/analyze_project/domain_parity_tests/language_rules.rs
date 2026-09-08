#[test]
fn prepared_check_matches_standalone_for_lexically_masked_language_rules() {
    let root = repo_fixture(&["fixtures", "rules", "swift-csharp-lexical-masking"]);
    let options = json!({ "root": root.clone(), "config": ".no-mistakes.yml" });
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            options.to_string(),
        ))
        .unwrap(),
    );
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "config": ".no-mistakes.yml",
                "reports": [{ "type": "check" }]
            })
            .to_string(),
        ))
        .unwrap(),
    );

    assert_eq!(aggregate["reports"][0]["result"], standalone);
    let rules = standalone["rules"].as_array().unwrap();
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "swift-no-raw-print"
            && finding["file"] == "SwiftPrint.swift"
            && finding["line"] == 12
    }));
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "csharp-no-async-void-delegate"
            && finding["file"] == "AsyncDelegate.cs"
            && finding["line"] == 11
    }));
    assert!(rules.iter().any(|finding| {
        finding["rule"] == "csharp-no-async-void-delegate"
            && finding["file"] == "AsyncDelegate.cs"
            && finding["line"] == 18
    }));
}
