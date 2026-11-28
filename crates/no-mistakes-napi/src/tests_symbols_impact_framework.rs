#[test]
fn signature_impact_cli_and_napi_share_configured_nonworkspace_framework_aliases() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/framework-project-alias");
    let fixture = no_mistakes::test_support::materialize_saved_fixture(&source);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(fixture.path());
    let cli = no_mistakes::codebase::symbols::run_json(no_mistakes::codebase::symbols::SymbolsArgs {
        files: vec![PathBuf::from("apps/web/src/project-options.ts")],
        root: Some(root.clone()),
        tsconfig: None,
        config: None,
        mode: no_mistakes::codebase::symbols::SymbolsMode::SignatureImpact,
        symbol: Some("projects".to_string()),
        kinds: Vec::new(),
        include: no_mistakes::codebase::symbols::Include::Exports,
        format: Some(no_mistakes::cli::Format::Json),
        json: true,
        timings: false,
    })
    .unwrap();
    let napi = symbols_json_impl(
        crate::options::test_json_arg(json!({
            "root": root,
            "files": ["apps/web/src/project-options.ts"],
            "mode": "signature-impact",
            "symbol": "projects"
        })
        .to_string(),)
    )
    .unwrap();
    let cli: serde_json::Value = serde_json::from_str(&cli).unwrap();
    let napi: serde_json::Value = serde_json::from_str(&napi).unwrap();

    assert_eq!(napi, cli);
    assert!(cli["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "apps/web/vitest.config.ts" && entry["symbol"] == "default"
    }), "{cli:#?}");
}

#[test]
fn signature_impact_uses_prepared_runner_projects_for_test_of_suggestions() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/framework-project-alias");
    let fixture = no_mistakes::test_support::materialize_saved_fixture(&source);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(fixture.path());
    let cli = no_mistakes::codebase::symbols::run_json(no_mistakes::codebase::symbols::SymbolsArgs {
        files: vec![PathBuf::from("apps/web/src/value.ts")],
        root: Some(root.clone()),
        tsconfig: None,
        config: None,
        mode: no_mistakes::codebase::symbols::SymbolsMode::SignatureImpact,
        symbol: Some("value".to_string()),
        kinds: Vec::new(),
        include: no_mistakes::codebase::symbols::Include::Exports,
        format: Some(no_mistakes::cli::Format::Json),
        json: true,
        timings: false,
    })
    .unwrap();
    let napi = symbols_json_impl(
        crate::options::test_json_arg(json!({
            "root": root,
            "files": ["apps/web/src/value.ts"],
            "mode": "signature-impact",
            "symbol": "value"
        })
        .to_string(),)
    )
    .unwrap();
    let cli: serde_json::Value = serde_json::from_str(&cli).unwrap();
    let napi: serde_json::Value = serde_json::from_str(&napi).unwrap();

    assert_eq!(napi, cli);
    assert!(cli["suggestedTests"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "apps/web/tests/value.impact.ts"
    }), "{cli:#?}");
}
