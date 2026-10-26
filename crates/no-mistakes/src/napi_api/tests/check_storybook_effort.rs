use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/require-storybook-stories/fixture/effort")
}

#[test]
fn public_storybook_effort_routes_residuals_and_retains_definite_findings() {
    let root = root();
    for (config, findings, advisories) in [
        (".no-mistakes.yml", 3, 0),
        (".no-mistakes.max.yml", 3, 0),
        (".no-mistakes.low.yml", 3, 0),
        (".no-mistakes.low-advise.yml", 1, 2),
        (".no-mistakes.low-ignore.yml", 1, 0),
    ] {
        let report: serde_json::Value = crate::test_support::policy_advisory_json(
            serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(
                    json!({ "root": root, "config": root.join(config) }).to_string(),
                ))
                .unwrap(),
            )
            .unwrap(),
            "require-storybook-stories",
        );
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{config}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}: {report}"
        );
        assert_eq!(report["warnings"], json!([]), "{report}");
    }
}

#[test]
fn analyze_project_storybook_effort_matches_check_and_overrides_rule_effort() {
    let root = root();
    for (effort, findings, advisories) in [("low", 1, 2), ("max", 3, 0)] {
        let check: serde_json::Value = crate::test_support::policy_advisory_json(serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({ "root": root, "config": root.join(".no-mistakes.low-advise.yml"), "effort": effort }).to_string(),
            )).unwrap(),
        ).unwrap(), "require-storybook-stories");
        let project: serde_json::Value = crate::test_support::policy_advisory_json(serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({ "root": root, "config": root.join(".no-mistakes.low-advise.yml"), "reports": [{ "type": "check", "effort": effort }] }).to_string(),
                ),
            ).unwrap(),
        ).unwrap(), "require-storybook-stories");
        assert_eq!(
            check["rules"].as_array().unwrap().len(),
            findings,
            "{check}"
        );
        assert_eq!(
            check["advisories"].as_array().unwrap().len(),
            advisories,
            "{check}"
        );
        assert_eq!(project["reports"][0]["result"], check, "{project}");
    }
}

#[test]
fn public_storybook_low_advisories_honor_file_and_line_suppression() {
    let root = root();
    let report: serde_json::Value = crate::test_support::policy_advisory_json(serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "config": root.join(".no-mistakes.low-advise.yml"), "includeSuppressed": true }).to_string(),
        )).unwrap(),
    ).unwrap(), "require-storybook-stories");
    assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
    assert_eq!(
        report["advisories"].as_array().unwrap().len(),
        2,
        "{report}"
    );
    assert_eq!(
        report["suppressed"].as_array().unwrap().len(),
        2,
        "{report}"
    );
}
