// Output cases moved to cli_postgres_snapshot_require_query_annotation.
use super::*;

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
