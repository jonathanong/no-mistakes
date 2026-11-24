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

        let explicit =
            crate::napi_api::queue_check_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root, "config": "explicit.yml" }).to_string(),
            ))
            .unwrap();
        let explicit: Value = serde_json::from_str(&explicit).unwrap();
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
            assert_eq!(queue_check, &explicit, "{directory}: {output}");
            let findings = queue_check.as_array().unwrap();
            assert_eq!(findings.len(), explicit_count, "{directory}: {output}");
            for finding in findings {
                assert_eq!(finding["kind"], "missing-job-option");
                assert_eq!(finding["message"], "job options missing attempts");
            }
        }
    }
}
