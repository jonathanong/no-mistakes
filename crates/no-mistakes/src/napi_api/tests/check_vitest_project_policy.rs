use super::*;

#[test]
fn vitest_project_policy_async_check_matches_cli() {
    for name in [
        "pass",
        "declared",
        "declared-filter",
        "declared-source-errors",
        "declared-line",
        "declared-next-line",
        "declared-file",
        "missing",
        "mismatch",
        "nonliteral",
        "helper",
        "required-forbidden",
        "helper-glob-skip",
        "helper-literal-file-brackets",
        "helper-literal-file-open-bracket",
        "helper-literal-dir-brackets",
        "helper-literal-dir-open-bracket",
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
        if name.starts_with("helper-") {
            let standalone =
                no_mistakes::codebase::rules::filesystem_dispatch::run_filesystem_rules(
                    &root,
                    Some(&root.join(".no-mistakes.yml")),
                )
                .unwrap();
            assert_eq!(standalone, cli.rules, "{name}");
            assert_eq!(
                standalone.len(),
                if name == "helper-glob-skip" { 3 } else { 2 },
                "{name}: {standalone:?}"
            );
            assert!(
                standalone
                    .iter()
                    .any(|finding| finding.message.contains("must equal 0")),
                "{name}"
            );
            assert!(
                standalone
                    .iter()
                    .any(|finding| finding.message.contains("declared.forbiddenValues")),
                "{name}"
            );
            assert!(native.get("incomplete").is_none(), "{name}: {native}");
            if name == "helper-glob-skip" {
                assert!(standalone
                    .iter()
                    .any(|finding| finding.file == "helpers/new-visible.ts"));
                assert_eq!(
                    standalone
                        .iter()
                        .filter(|finding| finding.file == "helpers/skipped/project.ts")
                        .count(),
                    2
                );
            }
        }
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
    for case in [
        "suppression",
        "line-suppression",
        "next-line-suppression",
        "declared-line",
        "declared-next-line",
        "declared-file",
    ] {
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

#[test]
fn declared_helper_filters_survive_deferred_suppression() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vitest-project-policy/fixture/declared-filter")
        .canonicalize()
        .unwrap();
    for include_suppressed in [false, true] {
        let options = json!({"root": root, "includeSuppressed": include_suppressed});
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
        assert_eq!(report["rules"][0]["file"], "helpers/included.mts");
        assert!(report.get("incomplete").is_none(), "{report}");
        if include_suppressed {
            let files = report["suppressed"]
                .as_array()
                .unwrap()
                .iter()
                .map(|finding| finding["file"].as_str().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                files,
                vec![
                    "helpers/suppressed-file.mts",
                    "helpers/suppressed-line.mts",
                    "helpers/suppressed-next.mts"
                ]
            );
        } else {
            assert!(report.get("suppressed").is_none());
        }
    }
}

#[test]
fn helper_glob_does_not_reopen_an_excluded_config() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vitest-project-policy/fixture/helper-glob-skip")
        .canonicalize()
        .unwrap();
    for config in ["config-excluded.yml", "config-skipped.yml"] {
        let config = root.join(config);
        let standalone = no_mistakes::codebase::rules::filesystem_dispatch::run_filesystem_rules(
            &root,
            Some(&config),
        )
        .unwrap();
        assert!(standalone.is_empty(), "{standalone:?}");
        let options = json!({"root": root, "config": config});
        let native: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(native["rules"], json!([]), "{native}");
        assert!(native.get("incomplete").is_none(), "{native}");
        let mut batched_options = options;
        batched_options["reports"] = json!([{"type": "check"}]);
        let batched: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(batched_options.to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(batched["reports"][0]["result"], native);
    }
}
