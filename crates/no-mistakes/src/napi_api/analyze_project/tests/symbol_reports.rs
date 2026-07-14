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
fn analyze_project_dispatches_signature_impact_symbols_report() {
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
