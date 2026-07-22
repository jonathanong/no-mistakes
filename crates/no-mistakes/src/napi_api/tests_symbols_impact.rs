#[test]
fn symbols_json_returns_signature_impact_report() {
    let options = json!({
        "root": fixture_root("tests-impact-symbol"),
        "files": ["utils.mts"],
        "mode": "signature-impact",
        "symbol": "parseDate"
    })
    .to_string();

    let output = symbols_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["symbol"], "parseDate");
    assert!(value["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "other.mts" && entry["symbol"] == "parse" }));
    assert!(value["suggestedTests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "other.test.mts" }));
}

#[test]
fn signature_impact_uses_the_importing_packages_tsconfig_alias_for_dependents() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/monorepo-tsconfig-catalog");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());

    let output = symbols_json_impl(
        json!({
            "root": root,
            "files": ["packages/lib/src/forbidden.ts"],
            "mode": "signature-impact",
            "symbol": "forbidden"
        })
        .to_string(),
    )
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(report["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["file"] == "packages/app/src/api.ts"), "{report:#?}");
}

#[test]
fn signature_impact_napi_parses_each_source_file_once() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/signature-impact"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();

    crate::ast::begin_parse_count(&root);
    let output = symbols_json_impl(
        json!({
            "root": root,
            "files": ["utils.mts"],
            "mode": "signature-impact",
            "symbol": "parseDate"
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = [
        root.join("consumer.mts"),
        root.join("consumer.test.mts"),
        root.join("utils.mts"),
    ];

    assert!(report["suggestedTests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["file"] == "consumer.test.mts"));
    assert_eq!(counts.len(), expected.len(), "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    for file in expected {
        assert_eq!(counts.get(&file), Some(&1), "{counts:?}");
    }
}

#[test]
fn pass4b_signature_impact_cli_and_napi_reports_share_gitignore_visibility() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let root_string = root.display().to_string();

    let cli_output = crate::codebase::symbols::run_json(
        crate::codebase::symbols::SymbolsArgs {
            files: vec![PathBuf::from("packages/pkg/src/feature.ts")],
            root: Some(root),
            tsconfig: None,
            config: None,
            mode: crate::codebase::symbols::SymbolsMode::SignatureImpact,
            symbol: Some("feature".to_string()),
            kinds: Vec::new(),
            include: crate::codebase::symbols::Include::Exports,
            format: Some(crate::cli::Format::Json),
            json: true,
            timings: false,
        },
    )
    .unwrap();
    let napi_output = symbols_json_impl(
        json!({
            "root": root_string,
            "files": ["packages/pkg/src/feature.ts"],
            "mode": "signature-impact",
            "symbol": "feature",
        })
        .to_string(),
    )
    .unwrap();
    let cli_output: serde_json::Value = serde_json::from_str(&cli_output).unwrap();
    let napi_output: serde_json::Value = serde_json::from_str(&napi_output).unwrap();

    assert_eq!(napi_output, cli_output);
    assert!(napi_output["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["file"] == "impact/importer.ts" && entry["symbol"] == "caller"));
    assert!(napi_output["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| {
            entry["file"] == "impact/namespace-consumer.ts"
                && entry["symbol"] == "namespaceCaller"
        }));
}

include!("tests_symbols_impact_framework.rs");
