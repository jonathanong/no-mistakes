use super::*;

#[test]
fn include_and_exclude_globs_filter_files() {
    let root = fixture("fail");
    let files = vec![ts_file(&root)];
    let excluded = check_with_files(
        &root,
        &config_with_options("exclude: ['src/query.ts']"),
        &files,
    )
    .unwrap();
    assert!(excluded.is_empty(), "{excluded:#?}");

    let included = check_with_files(
        &root,
        &config_with_options("include: ['src/query.ts']"),
        &files,
    )
    .unwrap();
    assert_eq!(included.len(), 1);

    let missed = check_with_files(
        &root,
        &config_with_options("include: ['src/missing.ts']"),
        &files,
    )
    .unwrap();
    assert!(missed.is_empty(), "{missed:#?}");
}

#[test]
fn custom_executor_and_specifier_are_required_to_match() {
    let root = fixture("fail");
    let files = vec![ts_file(&root)];
    let other = check_with_files(
        &root,
        &config_with_options("importSpecifier: '@other/db'\nexecutorNames: [run]"),
        &files,
    )
    .unwrap();
    assert!(other.is_empty(), "{other:#?}");
}

#[test]
fn invalid_include_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_with_options("include: ['[']"),
        &[ts_file(&root)],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn invalid_exclude_glob_errors() {
    let root = fixture("fail");
    let error = check_with_files(
        &root,
        &config_with_options("exclude: ['[']"),
        &[ts_file(&root)],
    )
    .expect_err("invalid glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("src/does-not-exist.ts");
    let error = check_with_files(&root, &default_config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect embedded SQL"),
        "{error}"
    );
}

#[test]
fn missing_sql_text_is_ignored() {
    let call = crate::codebase::postgres::EmbeddedSqlCall {
        variants: Vec::new(),
        line: 1,
        callee: "query".to_string(),
        sql_text: None,
        ..Default::default()
    };
    assert!(super::scan::findings_for_call("src/query.ts", &call).is_empty());
}

#[test]
fn compile_options_honor_overrides() {
    let compiled = compile_options(&Options {
        import_specifier: "@other/db".to_string(),
        executor_names: vec!["run".to_string()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
}

#[test]
fn compile_options_fill_defaults() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert_eq!(
        compiled.embedded.import_specifier,
        EmbeddedSqlOptions::default().import_specifier
    );
    assert_eq!(
        compiled.embedded.executor_names,
        EmbeddedSqlOptions::default().executor_names
    );
}
