use serde_json::Value;

#[test]
fn missing_path_argument_fact_has_a_distinct_reason() {
    let root =
        crate::check_json_snapshot::rules_path("test-no-live-repository-scan", "issue-3286", "");
    let report = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_issue_3286",
        &[("spread", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report: Value = report["spread"]["report"].clone();
    let advisories = report["advisories"].as_array().unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(advisories.len(), 4, "{report}");
    assert_eq!(findings.len(), 1, "{report}");
    for source_line in [
        "readdirSync(...missingPaths);",
        "readdirSync(...[]);",
        "readdirSync();",
    ] {
        let missing_argument = advisories
            .iter()
            .find(|finding| finding["sourceLine"] == source_line)
            .unwrap();
        assert!(missing_argument["message"]
            .as_str()
            .unwrap()
            .contains("path argument was not recorded (spread argument or unparsed module)"));
    }
    let unknown_value = advisories
        .iter()
        .find(|finding| finding["sourceLine"] == "readdirSync(unknownRoot);")
        .unwrap();
    assert!(!unknown_value["message"]
        .as_str()
        .unwrap()
        .contains("path argument was not recorded"));
    assert_eq!(unknown_value["target"], "unresolved root");
    assert_eq!(findings[0]["sourceLine"], "readdirSync(...livePaths);");
    assert_eq!(findings[0]["line"], 13);
}
