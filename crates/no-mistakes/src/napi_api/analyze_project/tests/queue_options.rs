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

#[test]
fn queue_unbounded_retention_uses_the_selected_config_in_prepared_reports() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-retention-config");
    for (directory, default_count, explicit_count) in [("default-on", 1, 0), ("default-off", 0, 1)]
    {
        let root = fixtures.join(directory);
        let standalone = crate::napi_api::queue_check_json_impl(
            crate::napi_api::options::test_json_arg(json!({ "root": root }).to_string()),
        )
        .unwrap();
        let standalone: Value = serde_json::from_str(&standalone).unwrap();
        assert_eq!(standalone.as_array().unwrap().len(), default_count);

        let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "config": root.join("explicit.yml"),
                "reports": [{ "type": "queueCheck" }, { "type": "check" }]
            })
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
            assert_eq!(finding["kind"], "unbounded-job-retention");
        }
    }
}

#[test]
fn queue_job_state_as_record_uses_the_selected_config_in_prepared_reports() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-job-state-config");
    for (directory, default_count, explicit_count) in [("default-on", 1, 0), ("default-off", 0, 1)]
    {
        let root = fixtures.join(directory);
        let standalone = crate::napi_api::queue_check_json_impl(
            crate::napi_api::options::test_json_arg(json!({ "root": root }).to_string()),
        )
        .unwrap();
        let standalone: Value = serde_json::from_str(&standalone).unwrap();
        assert_eq!(standalone.as_array().unwrap().len(), default_count);

        let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "config": root.join("explicit.yml"),
                "reports": [{ "type": "queueCheck" }, { "type": "check" }]
            })
            .to_string(),
        ))
        .unwrap();
        let output: Value = serde_json::from_str(&output).unwrap();
        let queue_check = output["reports"][0]["result"].as_array().unwrap();
        let aggregate_queues = output["reports"][1]["result"]["queues"].as_array().unwrap();
        let advisories = output["reports"][1]["result"]["advisories"]
            .as_array()
            .unwrap();
        assert!(aggregate_queues.is_empty(), "{directory}: {output}");
        assert_eq!(queue_check.len(), explicit_count, "{directory}: {output}");
        assert_eq!(advisories.len(), explicit_count, "{directory}: {output}");
        for finding in queue_check {
            assert_eq!(finding["kind"], "job-state-as-record");
            assert_eq!(finding["severity"], "advisory");
        }
        for advisory in advisories {
            assert_eq!(advisory["rule"], "queues-check");
        }
    }
}

#[test]
fn queue_shared_test_scope_respects_include_tests_in_prepared_reports() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-shared-test-scope");
    for (config, expected_retention, expected_advisories) in [
        (None, 1usize, 1usize),
        (Some(root.join("include-test-file.yml")), 2usize, 2usize),
    ] {
        let mut request = json!({
            "root": root,
            "reports": [{ "type": "queueCheck" }, { "type": "check" }]
        });
        if let Some(config) = config {
            request["config"] = json!(config);
        }
        let output =
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(request.to_string()))
                .unwrap();
        let output: Value = serde_json::from_str(&output).unwrap();
        let queue_check = output["reports"][0]["result"].as_array().unwrap();
        let aggregate_queues = output["reports"][1]["result"]["queues"].as_array().unwrap();
        let advisories = output["reports"][1]["result"]["advisories"]
            .as_array()
            .unwrap();
        assert_eq!(aggregate_queues.len(), expected_retention, "{output}");
        assert_eq!(advisories.len(), expected_advisories, "{output}");
        assert_eq!(
            queue_check.len(),
            expected_retention + expected_advisories,
            "{output}"
        );
    }
}

#[test]
fn queue_retention_call_site_warnings_match_in_prepared_reports() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-retention-call-site");
    let standalone = standalone_check_result(&root);
    let prepared = analyze_project_check_result(&root);
    assert_eq!(
        standalone["queues"].as_array().unwrap().len(),
        3,
        "{standalone}"
    );
    assert_eq!(prepared["queues"], standalone["queues"]);
    let warnings = standalone["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 9, "{standalone}");
    for line in [12, 13, 16] {
        assert!(
            warnings.iter().any(|warning| {
                let warning = warning.as_str().unwrap();
                warning.contains(&format!("enqueue.ts:{line}"))
                    && warning.contains("debounce-ttl was not judged")
            }),
            "{line}: {warnings:?}"
        );
    }
    assert_eq!(prepared["warnings"], standalone["warnings"]);
}
