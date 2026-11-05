use super::*;

#[test]
fn public_check_and_project_preserve_rule_recall_and_suppression() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rule-recall-audit/fixture");
    for (case, config, findings, advisories) in [
        ("workflow/missing-edge", "config.json", 1, 0),
        ("workflow/line-suppression", "config.json", 0, 0),
        ("live-scan/missing-relative", ".no-mistakes.yml", 0, 1),
        ("live-scan/absolute-control", ".no-mistakes.yml", 0, 1),
        ("storybook/empty-stories", ".no-mistakes.yml", 1, 0),
        ("storybook/omitted-stories", ".no-mistakes.yml", 0, 0),
        (
            "storybook/required-props-explicit",
            ".no-mistakes.yml",
            0,
            0,
        ),
        (
            "download-integrity/quoted-backtick",
            ".no-mistakes.yml",
            1,
            0,
        ),
        ("timeout-literals/multiline", ".no-mistakes.yml", 1, 0),
        ("child-links/next-line", ".no-mistakes.yml", 0, 0),
    ] {
        let root = base.join(case);
        let options = json!({ "root": root, "config": root.join(config) });
        let check: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(
            check["rules"].as_array().unwrap().len(),
            findings,
            "{case}: {check}"
        );
        assert_eq!(
            check["advisories"].as_array().unwrap().len(),
            advisories,
            "{case}: {check}"
        );
        let mut options = options;
        options["reports"] = json!([{ "type": "check" }]);
        let project: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(
                &crate::napi_api::analyze_project::analyze_project_json_impl(
                    crate::napi_api::options::test_json_arg(options.to_string()),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(project["reports"][0]["result"], check, "{case}: {project}");
    }
}
