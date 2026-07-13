fn impact_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    )
}

fn impact_args(symbol: &str, format: Format) -> SymbolsArgs {
    impact_file_args("utils.mts", symbol, format)
}

fn impact_file_args(file: &str, symbol: &str, format: Format) -> SymbolsArgs {
    let mut args = args_for(&impact_fixture_root(), vec![file], format);
    args.mode = SymbolsMode::SignatureImpact;
    args.symbol = Some(symbol.to_string());
    args
}

fn signature_parser_count_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/signature-impact"),
    )
}

#[test]
fn signature_impact_parses_each_source_file_once() {
    let source = signature_parser_count_fixture_root();
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let mut args = args_for(&root, vec!["utils.mts"], Format::Json);
    args.mode = SymbolsMode::SignatureImpact;
    args.symbol = Some("parseDate".to_string());

    crate::ast::begin_parse_count(&root);
    let output = impact::report_json(args).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = [
        root.join("consumer.mts"),
        root.join("consumer.test.mts"),
        root.join("utils.mts"),
    ];

    assert!(report["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["file"] == "consumer.mts"));
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
fn signature_impact_json_groups_callers_exports_and_tests() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["symbol"], "parseDate");
    assert_eq!(v["definition"]["file"], "utils.mts");
    assert_eq!(v["definition"]["line"], 2);
    assert!(v["exports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "date-barrel.mts" && entry["kind"] == "re-export" }));
    assert!(v["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "other.mts" && entry["symbol"] == "parse" }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "barrel-consumer.mts" && entry["symbol"] == "parsePublicDate"
    }));
    assert!(v["testCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "helper-export.test.mts" && entry["symbol"] == "helper" }));
    assert!(!v["testCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "other.test.mts" }));
}

#[test]
fn signature_impact_paths_emit_suggested_tests_only() {
    let out = run_capture(impact_args("parseDate", Format::Paths));
    let lines: Vec<_> = out.lines().collect();

    assert!(lines.contains(&"helper-export.test.mts"));
    assert!(lines.contains(&"other.test.mts"));
    assert!(!lines.contains(&"other.mts"));
}

#[test]
fn signature_impact_text_formats_describe_callers_and_tests() {
    let human = run_capture(impact_args("parseDate", Format::Human));
    assert!(human.contains("Symbol: parseDate"));
    assert!(human.contains("Defined in: utils.mts:2"));
    assert!(human.contains("## Exported via"));
    assert!(human.contains("- `other.mts#parse`"));
    assert!(human.contains("Suggested tests:"));
    assert!(human.contains("  other.test.mts"));

    let markdown = run_capture(impact_args("parseDate", Format::Md));
    assert!(markdown.contains("# `utils.mts#parseDate`"));
    assert!(markdown.contains("- Defined in: `utils.mts` (line 2)"));
    assert!(markdown.contains("- `date-barrel.mts#parseDate` (re-export, line 1)"));
    assert!(markdown.contains("- `helper-export.test.mts#helper`"));
    assert!(markdown.contains("- `other.test.mts`"));
}

#[test]
fn signature_impact_text_formats_include_file_callers() {
    let human = run_capture(impact_file_args("alpha-source.mts", "alpha", Format::Human));

    assert!(human.contains("- `alpha-consumer.test.mts`"));
    assert!(human.contains("  alpha-barrel.test.mts"));
}

#[test]
fn signature_impact_text_formats_show_empty_sections() {
    let human = run_capture(impact_file_args("alpha-source.mts", "beta", Format::Human));
    assert!(human.contains("## Production callers\n_None._"));
    assert!(human.contains("## Test callers\n_None._"));
    assert!(human.contains("Suggested tests:\n  (none)"));

    let markdown = run_capture(impact_file_args("alpha-source.mts", "beta", Format::Md));
    assert!(markdown.contains("## Production callers\n_None._"));
    assert!(markdown.contains("## Test callers\n_None._"));
    assert!(markdown.contains("## Suggested tests\n_No suggested tests found._"));
}

#[test]
fn signature_impact_yml_and_json_helpers_emit_structured_report() {
    let yml = run_capture(impact_args("parseDate", Format::Yml));
    assert!(yml.contains("symbol: parseDate"));
    assert!(yml.contains("suggestedTests:"));
    assert!(yml.contains("file: other.test.mts"));

    let json = impact::report_json(impact_args("parseDate", Format::Human)).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["symbol"], "parseDate");
    assert_eq!(v["roots"][0], "utils.mts#parseDate");
}

#[test]
fn signature_impact_warns_when_no_tests_are_reachable() {
    let out = run_capture(impact_file_args("alpha-source.mts", "beta", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["suggestedTests"].as_array().unwrap().len(), 0);
    assert_eq!(v["warnings"][0]["type"], "no-suggested-tests");
    assert!(v["warnings"][0]["message"]
        .as_str()
        .unwrap()
        .contains("No test files were reachable"));
}

#[test]
fn signature_impact_supports_default_exports() {
    let out = run_capture(impact_file_args("default-source.mts", "default", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["definition"]["symbol"], "default");
    assert_eq!(v["definition"]["kind"], "default");
}

#[test]
fn signature_impact_tracks_star_reexport_paths_and_consumers() {
    let out = run_capture(impact_args("parseDate", Format::Json));
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert!(v["exports"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "star-date-barrel.mts" && entry["symbol"] == "parseDate"
    }));
    assert!(v["productionCallers"].as_array().unwrap().iter().any(|entry| {
        entry["file"] == "star-consumer.mts" && entry["symbol"] == "parseStarDate"
    }));
}

include!("tests_signature_impact_barrels.rs");
