use super::*;

#[test]
fn vitest_project_policy_async_check_matches_cli() {
    for name in [
        "pass",
        "missing",
        "mismatch",
        "nonliteral",
        "helper",
        "required-forbidden",
    ] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-policy/fixture")
            .join(name)
            .canonicalize()
            .unwrap();
        let options = json!({"root": root, "config": root.join(".no-mistakes.yml")});
        let native: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap();
        let cli = crate::check_runner::run_all_with_options(
            root.clone(),
            Some(root.join(".no-mistakes.yml")),
            None,
            false,
            false,
            crate::check_runner::AnalysisOptions::default(),
        )
        .unwrap();
        let cli = crate::check_runner::json_value(&cli);
        assert_eq!(native, cli, "{name}");
        let mut batched_options = options.clone();
        batched_options["reports"] = json!([{"type": "check"}]);
        let batched: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(batched_options.to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(batched["reports"][0]["result"], native, "{name}");
        if name == "nonliteral" {
            assert!(native["incomplete"]
                .as_array()
                .unwrap()
                .iter()
                .any(|failure| { failure["consumer"] == "vitest-project-policy" }));
        }
    }
}

#[test]
fn vitest_policy_suppression_survives_prepared_check_reuse() {
    for case in ["suppression", "line-suppression", "next-line-suppression"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-policy/fixture")
            .join(case)
            .canonicalize()
            .unwrap();
        for include_suppressed in [false, true] {
            let options = json!({"root": root, "includeSuppressed": include_suppressed});
            let report: serde_json::Value = serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap();
            let cli = crate::check_runner::run_all_with_options(
                root.clone(),
                Some(root.join(".no-mistakes.yml")),
                None,
                include_suppressed,
                false,
                crate::check_runner::AnalysisOptions::default(),
            )
            .unwrap();
            assert_eq!(report, crate::check_runner::json_value(&cli), "{case}");
            assert_eq!(report["rules"], json!([]), "{case}: {report}");
            if include_suppressed {
                assert_eq!(
                    report["suppressed"].as_array().unwrap().len(),
                    1,
                    "{case}: {report}"
                );
            } else {
                assert!(report.get("suppressed").is_none(), "{case}: {report}");
            }
        }
    }
}
