#[test]
fn report_options_forward_relative_top_level_config_from_root() {
    let root = fixture_root("simple");
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": root,
            "config": "no-mistakes.json",
            "reports": [{
                "type": "symbols",
                "root": "nested",
                "files": ["a.mts"],
            }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(symbols["config"], format!("{root}/no-mistakes.json"));
}

#[test]
fn report_options_keep_per_report_config_override() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "config": "no-mistakes.json",
            "reports": [{
                "type": "symbols",
                "files": ["a.mts"],
                "config": "custom.json"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(symbols["config"], "custom.json");
}

#[test]
fn report_options_keep_relative_top_level_config_without_root() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "config": "no-mistakes.json",
            "reports": [{
                "type": "symbols",
                "root": "nested",
                "files": ["a.mts"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let symbols: Value =
        serde_json::from_str(&options::symbols_options(&options.reports[0], &options).unwrap())
            .unwrap();
    assert_eq!(
        symbols["config"],
        std::env::current_dir()
            .unwrap()
            .join("no-mistakes.json")
            .display()
            .to_string()
    );
}
