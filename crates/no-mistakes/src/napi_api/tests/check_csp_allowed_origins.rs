use super::*;

#[test]
fn csp_allowed_origins_async_check_matches_cli() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/report")
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
    assert_eq!(cli.rules.len(), 7);
    assert!(cli
        .rules
        .iter()
        .all(|finding| finding.rule == "csp-allowed-origins"));
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

    let advise_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/csp-allowed-origins/fixture/unanalyzable-and-urls")
        .canonicalize()
        .unwrap();
    let advise_config = advise_root.join("advise.yml");
    let advise_opts = json!({"root": advise_root, "config": advise_config});
    let advise_native: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            advise_opts.to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let advise_cli = crate::check_runner::run_all_with_options(
        advise_root,
        Some(advise_config),
        None,
        false,
        false,
        crate::check_runner::AnalysisOptions::default(),
    )
    .unwrap();
    assert_eq!(advise_cli.rules.len(), 2);
    assert_eq!(advise_cli.advisories.len(), 5);
    assert_eq!(advise_native, crate::check_runner::json_value(&advise_cli));
}
