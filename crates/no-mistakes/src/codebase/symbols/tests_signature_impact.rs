fn impact_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    )
}

fn impact_args(symbol: &str, format: Format) -> SymbolsArgs {
    let mut args = args_for(&impact_fixture_root(), vec!["utils.mts"], format);
    args.mode = SymbolsMode::SignatureImpact;
    args.symbol = Some(symbol.to_string());
    args
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
    assert!(v["testCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| { entry["file"] == "helper-export.test.mts" && entry["symbol"] == "helper" }));
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
fn signature_impact_validates_symbol_and_file_count() {
    let mut missing_symbol = impact_args("parseDate", Format::Json);
    missing_symbol.symbol = None;
    let err = impact::collect_report(&missing_symbol).unwrap_err();
    assert!(err.to_string().contains("requires --symbol"));

    let mut multiple_files = impact_args("parseDate", Format::Json);
    multiple_files.files.push(PathBuf::from("other.mts"));
    let err = impact::collect_report(&multiple_files).unwrap_err();
    assert!(err.to_string().contains("exactly one file"));

    let err = impact::collect_report(&impact_args("missing", Format::Json)).unwrap_err();
    assert!(err.to_string().contains("is not exported"));
}
