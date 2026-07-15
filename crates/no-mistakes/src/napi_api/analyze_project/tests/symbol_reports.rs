#[test]
fn analyze_project_shared_dependencies_uses_symbol_graph_when_included() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "dependencies",
                "id": "deps",
                "includeSymbols": true,
                "relationships": ["import"],
                "files": [{ "file": "other.mts", "symbol": "parse" }]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"].as_array().unwrap();

    assert!(files
        .iter()
        .any(|file| file["file"] == "utils.mts" && file["symbol"] == "parseDate"));
}

#[test]
fn analyze_project_list_symbols_matches_legacy_standalone_parse_semantics_once() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/symbols-output/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let files = [
        "src/recoverable-diagnostic.mts",
        "src/types-in-js.js",
        "src/types-in-mjs.mjs",
    ];
    let standalone = crate::napi_api::symbols_json_impl(
        json!({ "root": root, "files": files }).to_string(),
    )
    .unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();

    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [{ "type": "symbols", "files": files }]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let output: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(output["reports"][0]["result"], standalone);
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    for file in files {
        assert_eq!(counts.get(&root.join(file)), Some(&1), "{counts:#?}");
    }

    let error = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [{ "type": "symbols", "files": ["src/invalid.mts"] }]
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(error.reason.contains("failed to parse TypeScript source"));
}

#[test]
fn analyze_project_shared_symbol_graph_does_not_leak_into_plain_reports() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [
                {
                    "type": "dependencies",
                    "id": "symbol-deps",
                    "includeSymbols": true,
                    "relationships": ["import"],
                    "files": [{ "file": "other.mts", "symbol": "parse" }]
                },
                {
                    "type": "dependencies",
                    "id": "plain-deps",
                    "relationships": ["import"],
                    "files": ["other.mts"]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][1]["result"]["files"].as_array().unwrap();

    assert!(files.iter().any(|file| file["path"] == "utils.mts"));
    assert!(!files.iter().any(|file| file.get("symbol").is_some()));
}

#[test]
fn analyze_project_shared_dependents_uses_symbol_graph_when_included() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "dependents",
                "id": "users",
                "includeSymbols": true,
                "relationships": ["import"],
                "files": [{ "file": "utils.mts", "symbol": "parseDate" }]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"].as_array().unwrap();

    assert!(files
        .iter()
        .any(|file| file["file"] == "other.mts" && file["symbol"] == "parse"));
    assert!(!files
        .iter()
        .any(|file| file["path"] == "unrelated-consumer.mts"));
}

#[test]
fn analyze_project_symbol_entrypoints_match_standalone_without_symbol_output() {
    let root = fixture_root("tests-impact-symbol");
    let entrypoints = [
        json!("utils.mts#parseDate"),
        json!({ "file": "utils.mts", "symbol": "parseDate" }),
    ];

    for entrypoint in entrypoints {
        let options = json!({
            "root": root,
            "files": [entrypoint],
            "includeSymbols": false,
            "relationships": ["import"]
        });
        let standalone = crate::napi_api::dependents_json_impl(options.to_string()).unwrap();
        let output = analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [{
                    "type": "dependents",
                    "files": options["files"],
                    "includeSymbols": false,
                    "relationships": ["import"]
                }]
            })
            .to_string(),
        )
        .unwrap();
        let standalone: Value = serde_json::from_str(&standalone).unwrap();
        let output: Value = serde_json::from_str(&output).unwrap();
        let result = &output["reports"][0]["result"];

        assert_eq!(result, &standalone);
        assert!(result["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["path"] == "other.mts"));
    }
}

#[test]
fn analyze_project_dispatches_signature_impact_symbols_report() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("tests-impact-symbol"),
            "reports": [{
                "type": "symbols",
                "id": "impact",
                "files": ["utils.mts"],
                "mode": "signature-impact",
                "symbol": "parseDate"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let result = &value["reports"][0]["result"];

    assert_eq!(value["reports"][0]["id"], "impact");
    assert_eq!(result["symbol"], "parseDate");
    assert!(result["testCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "helper-export.test.mts" }));

    let work = observer.snapshot().work;
    assert_ne!(work["resolver.computations"], 0, "{work:#?}");
    assert_eq!(
        work["resolver.computations"], work["resolver.unique_keys"],
        "{work:#?}"
    );
    // This fixture intentionally has both a root package.json and a workspace
    // package.json. Canonical preparation parses those two manifests plus the
    // config and tsconfig exactly once; a second workspace load would surface
    // as cache hits and weaken the one-pass invariant protected here.
    let manifest_cache_hits = work.get("manifest.cache_hits").copied().unwrap_or(0);
    assert_eq!(manifest_cache_hits, 0, "{work:#?}");
    assert_eq!(work["manifest.requests"], 4, "{work:#?}");
    assert_eq!(work["manifest.parses"], 4, "{work:#?}");
    assert_eq!(
        work["manifest.requests"],
        work["manifest.parses"] + manifest_cache_hits,
    );
}

#[test]
fn analyze_project_signature_impact_validates_prepared_inputs() {
    let root = fixture_root("tests-impact-symbol");
    let cases = [
        (
            json!({
                "type": "symbols",
                "files": ["utils.mts", "other.mts"],
                "mode": "signature-impact",
                "symbol": "parseDate"
            }),
            "exactly one file",
        ),
        (
            json!({
                "type": "symbols",
                "files": ["utils.mts"],
                "mode": "signature-impact"
            }),
            "requires --symbol",
        ),
    ];

    for (report, expected) in cases {
        let error = analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [report]
            })
            .to_string(),
        )
        .unwrap_err();

        assert!(error.to_string().contains(expected), "{error:#}");
    }
}

#[test]
fn analyze_project_shared_symbol_dependents_expands_file_roots() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("symbol-export"),
            "reports": [{
                "type": "dependents",
                "id": "users",
                "includeSymbols": true,
                "relationships": ["import"],
                "files": ["file-root-source.mts"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"].as_array().unwrap();

    assert!(files
        .iter()
        .any(|file| file["path"] == "file-root-consumer.mts"));
    assert!(files
        .iter()
        .any(|file| file["file"] == "file-root-consumer.mts" && file["symbol"] == "value"));
}

#[test]
fn tests_impact_api_requires_entrypoints() {
    let error = crate::napi_api::cli_parity::build_impact_args(
        crate::napi_api::options::TestsImpactOptions {
            entrypoints: vec![],
            include_symbols: false,
            root: None,
            config: None,
            tsconfig: None,
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("entrypoints is required"));
}
