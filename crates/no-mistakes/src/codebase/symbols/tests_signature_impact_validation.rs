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

    let err =
        impact::collect_report(&impact_file_args("unused-import.mts", "parseDate", Format::Json))
            .unwrap_err();
    assert!(err.to_string().contains("is not exported"));
}

#[test]
fn signature_impact_surfaces_tsconfig_and_config_errors() {
    let root = fixture_root();
    let mut invalid_tsconfig = args_for(&root, vec!["src/utils.mts"], Format::Json);
    invalid_tsconfig.mode = SymbolsMode::SignatureImpact;
    invalid_tsconfig.symbol = Some("foo".to_string());
    invalid_tsconfig.tsconfig = Some(root.join("tsconfig-invalid.json"));

    let error = impact::collect_report(&invalid_tsconfig).unwrap_err();
    let detail = format!("{error:#}");
    assert!(detail.contains("loading tsconfig"), "{detail}");
    assert!(detail.contains("tsconfig-invalid.json"), "{detail}");
    assert!(
        detail.contains("Unexpected close brace on line 4 column 3"),
        "{detail}"
    );

    let mut invalid_config = args_for(&root, vec!["src/utils.mts"], Format::Json);
    invalid_config.mode = SymbolsMode::SignatureImpact;
    invalid_config.symbol = Some("foo".to_string());
    invalid_config.config = Some(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/config-v2/invalid-rule-path-filter/fixture/.no-mistakes.yml",
        ),
    );

    let error = impact::collect_report(&invalid_config).unwrap_err();
    let detail = format!("{error:#}");
    assert!(
        detail.contains("rules[0].exclude contains invalid glob `[`"),
        "{detail}"
    );
    assert!(detail.contains("unclosed character class"), "{detail}");
}
