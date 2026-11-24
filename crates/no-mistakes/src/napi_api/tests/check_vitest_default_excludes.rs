use super::*;

#[test]
fn vitest_default_excludes_async_check_matches_cli() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vitest-default-excludes/fixture/report")
        .canonicalize()
        .unwrap();
    let config = root.join(".no-mistakes.yml");
    let mut options = json!({"root": root, "config": config});
    let native: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap();
    let cli = crate::check_runner::run_all_with_options(
        root,
        Some(config),
        None,
        false,
        false,
        crate::check_runner::AnalysisOptions::default(),
    )
    .unwrap();
    assert_eq!(cli.rules.len(), 2);
    assert!(cli
        .rules
        .iter()
        .all(|finding| finding.rule == "vitest-default-excludes"));
    assert_eq!(native, crate::check_runner::json_value(&cli));

    options["reports"] = json!([{"type": "check"}]);
    let batched: serde_json::Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(options.to_string()),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batched["reports"][0]["result"], native);
}

#[test]
fn vitest_default_excludes_supplemental_configs_keep_batch_parity() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vitest-default-excludes/fixture/supplemental")
        .canonicalize()
        .unwrap();
    let config = root.join(".no-mistakes.yml");
    let dependents = json!({"type": "dependents", "files": ["shared.ts"]});
    let baseline = batch(json!({"root": root, "config": config, "reports": [dependents]}));
    assert_eq!(
        baseline["reports"][0]["result"]["files"][0]["path"],
        "consumer.ts"
    );

    for include_deadlines in [false, true] {
        let options = json!({
            "root": root,
            "config": config,
            "includeRunnerConfigDeadlines": include_deadlines,
        });
        let native: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap();
        let cli = crate::check_runner::run_all_with_evidence(
            root.clone(),
            Some(config.clone()),
            None,
            false,
            include_deadlines,
        )
        .unwrap();
        assert_eq!(native, crate::check_runner::json_value(&cli));
        assert_eq!(native["rules"].as_array().unwrap().len(), 1);
        assert_eq!(native["rules"][0]["file"], "skipped/vitest.config.ts");
        assert!(native["incomplete"]
            .as_array()
            .unwrap()
            .iter()
            .any(|failure| {
                failure["consumer"] == "vitest-default-excludes"
                    && failure["file"] == "missing.config.ts"
                    && failure["property"] == "test.exclude"
                    && failure["message"]
                        .as_str()
                        .unwrap()
                        .contains("failed to read")
            }));

        crate::ast::begin_parse_count(&root);
        let mixed = batch(json!({
            "root": root,
            "config": config,
            "reports": [dependents, {
                "type": "check",
                "includeRunnerConfigDeadlines": include_deadlines,
            }],
        }));
        let counts = crate::ast::finish_parse_count(&root);
        assert!(!counts.is_empty());
        assert_eq!(
            counts.get(&root.join("skipped/vitest.config.ts")),
            Some(&1),
            "includeRunnerConfigDeadlines={include_deadlines}: {counts:?}"
        );
        assert_eq!(
            mixed["reports"][0]["result"],
            baseline["reports"][0]["result"]
        );
        assert_eq!(mixed["reports"][1]["result"], native);
    }
}

#[test]
fn vitest_default_excludes_files_option_matches_cli_and_batch() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vitest-default-excludes/fixture/factories")
        .canonicalize()
        .unwrap();
    let config = root.join(".no-mistakes.yml");
    let mut options = json!({"root": root, "config": config});
    let native: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap();
    let cli = crate::check_runner::run_all_with_options(
        root.clone(),
        Some(config.clone()),
        None,
        false,
        false,
        crate::check_runner::AnalysisOptions::default(),
    )
    .unwrap();
    assert_eq!(cli.rules.len(), 1);
    assert_eq!(cli.rules[0].rule, "vitest-default-excludes");
    assert_eq!(cli.rules[0].file, "config/vitest-bad-factory.ts");
    assert_eq!(cli.rules[0].line, 4);
    assert_eq!(native, crate::check_runner::json_value(&cli));

    crate::ast::begin_parse_count(&root);
    options["reports"] = json!([{"type": "check"}]);
    let batched = batch(options);
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(
        counts.get(&root.join("config/vitest-bad-factory.ts")),
        Some(&1),
        "{counts:?}"
    );
    assert_eq!(
        counts.get(&root.join("config/vitest-good-factory.ts")),
        Some(&1),
        "{counts:?}"
    );
    assert_eq!(counts.get(&root.join("other/unchecked-factory.ts")), None);
    assert_eq!(batched["reports"][0]["result"], native);
}

fn batch(options: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(options.to_string()),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn invalid_rule_options_reject_with_cli_and_batch_parity() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/options-audit/fixture/issue-3958");
    for (config_name, expected_lines) in [
        ("cache.yml", 1usize),
        ("action-input.yml", 1),
        ("vitest.yml", 1),
        ("multiple.yml", 11),
    ] {
        let config = root.join(config_name);
        let cli_err = crate::check_runner::run_all_with_options(
            root.clone(),
            Some(config.clone()),
            None,
            false,
            false,
            crate::check_runner::AnalysisOptions::default(),
        )
        .err()
        .expect("invalid rule options must fail CLI check");
        let cli_message = format!("{cli_err:#}");

        let options = json!({"root": root, "config": config});
        let native_err =
            check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                .expect_err("invalid rule options must reject async check");
        assert_eq!(native_err.reason, cli_message, "{config_name}");

        let batched_err = crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": config, "reports": [{"type": "check"}]}).to_string(),
            ),
        )
        .expect_err("invalid rule options must reject analyzeProject check");
        assert_eq!(batched_err.reason, cli_message, "{config_name}");

        let lines: Vec<_> = cli_message.lines().collect();
        assert_eq!(lines.len(), expected_lines, "{config_name}: {cli_message}");
        let mut sorted = lines.clone();
        sorted.sort();
        assert_eq!(lines, sorted, "{config_name}: {cli_message}");
    }
}
