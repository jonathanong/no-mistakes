use super::*;

#[test]
fn queue_required_options_use_the_selected_config_in_prepared_reports() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-required-config");
    for (directory, default_count, explicit_count) in [("default-on", 1, 0), ("default-off", 0, 1)]
    {
        let root = fixtures.join(directory);
        let standalone = crate::napi_api::queue_check_json_impl(
            crate::napi_api::options::test_json_arg(json!({ "root": root }).to_string()),
        )
        .unwrap();
        let standalone: Value = serde_json::from_str(&standalone).unwrap();
        assert_eq!(standalone.as_array().unwrap().len(), default_count);

        for traversal in [false, true] {
            let mut reports = vec![json!({ "type": "queueCheck" }), json!({ "type": "check" })];
            if traversal {
                reports.push(json!({ "type": "queueEdges" }));
            }
            let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root, "config": root.join("explicit.yml"), "reports": reports })
                    .to_string(),
            ))
            .unwrap();
            let output: Value = serde_json::from_str(&output).unwrap();
            let queue_check = &output["reports"][0]["result"];
            let aggregate = &output["reports"][1]["result"]["queues"];
            assert_eq!(queue_check, aggregate, "{directory}: {output}");
            let findings = queue_check.as_array().unwrap();
            assert_eq!(findings.len(), explicit_count, "{directory}: {output}");
            for finding in findings {
                assert_eq!(finding["kind"], "missing-job-option");
                assert_eq!(finding["message"], "job options missing attempts");
            }
        }
    }
}

#[test]
fn queue_policies_match_standalone_and_analyze_project() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/queue-policy-config");
    let input = |options: Value| crate::napi_api::options::test_json_arg(options.to_string());
    let standalone =
        crate::napi_api::queue_check_json_impl(input(json!({ "root": root }))).unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();
    let findings = standalone.as_array().unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .any(|finding| finding["kind"] == "unbounded-job-retention"));
    assert!(findings
        .iter()
        .any(|finding| finding["kind"] == "queue-job-state-as-record"
            && finding["severity"] == "advisory"));

    let reports = json!([{ "type": "queueCheck" }, { "type": "check" }, { "type": "queueEdges" }]);
    for (config, expected) in [
        (".no-mistakes.yml", standalone.clone()),
        (
            "advisory.yml",
            Value::Array(
                findings
                    .iter()
                    .filter(|finding| finding["severity"] == "advisory")
                    .cloned()
                    .collect(),
            ),
        ),
        ("off.yml", json!([])),
    ] {
        let output = analyze_project_json_impl(input(json!({
            "root": root,
            "config": root.join(config),
            "reports": reports,
        })))
        .unwrap();
        let output: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            output["reports"][0]["result"], expected,
            "{config}: {output}"
        );
        assert_eq!(
            output["reports"][1]["result"]["queues"], expected,
            "{config}: {output}"
        );
    }
}
