use super::*;

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
fn custom_safe_directive_is_honored() {
    let root = fixture("custom-directive");
    let file = ts_file(&root);
    let findings = check_with_files(
        &root,
        &config_with_options("safeDirective: ordered-locks"),
        &[file],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn sql_comment_directive_suppresses() {
    assert!(findings_for("pass-sql-comment").is_empty());
}

#[test]
fn distant_directive_does_not_suppress() {
    let findings = findings_for("too-far");
    assert_eq!(findings.len(), 1, "{findings:#?}");
}

#[test]
fn non_locking_sql_is_ignored() {
    assert!(findings_for("no-lock").is_empty());
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
    let compiled = compile_options(&Options::default()).unwrap();
    let call = crate::codebase::postgres::EmbeddedSqlCall {
        variants: Vec::new(),
        line: 1,
        callee: "query".to_string(),
        sql_text: None,
        ..Default::default()
    };
    assert!(findings_for_call("src/lock.ts", "", &call, &compiled, None).is_empty());
}

#[test]
fn compile_options_honor_overrides() {
    let compiled = compile_options(&Options {
        import_specifier: "@other/db".to_string(),
        executor_names: vec!["write".to_string(), "run".to_string(), "write".to_string()],
        safe_directive: "ordered-locks".to_string(),
        schema_catalog_path: "schema.json".to_string(),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run", "write"]);
    assert_eq!(compiled.safe_directive, "ordered-locks");
    assert_eq!(compiled.schema_catalog_path.as_deref(), Some("schema.json"));
}

#[test]
fn empty_safe_directive_does_not_match_everything() {
    assert!(!has_safe_directive(
        "/* deadlock-safe */\nquery(`SELECT 1 FOR UPDATE`)\n",
        2,
        "SELECT 1 FOR UPDATE",
        "",
    ));
}

#[test]
fn lookback_handles_line_one_and_unclosed_block_comment() {
    assert!(has_safe_directive(
        "/* deadlock-safe: same line */ query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`)",
        1,
        "SELECT * FROM t WHERE id = ANY($1) FOR UPDATE",
        DEFAULT_SAFE_DIRECTIVE,
    ));
    assert!(comment_contains_directive(
        "/* deadlock-safe without closer",
        DEFAULT_SAFE_DIRECTIVE,
    ));
    assert!(comment_contains_directive(
        "-- deadlock-safe line comment",
        DEFAULT_SAFE_DIRECTIVE,
    ));
    assert!(!comment_contains_directive(
        "deadlock-safe outside a comment",
        DEFAULT_SAFE_DIRECTIVE,
    ));
    assert!(!comment_contains_directive(
        "/* other */ leftover",
        DEFAULT_SAFE_DIRECTIVE,
    ));
    assert!(!comment_contains_directive(
        "-- other\nleftover",
        DEFAULT_SAFE_DIRECTIVE,
    ));
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
    assert_eq!(compiled.safe_directive, DEFAULT_SAFE_DIRECTIVE);
}

#[test]
fn contains_for_update_is_case_insensitive() {
    assert!(contains_for_update("select * from t for update"));
    assert!(!contains_for_update("SELECT * FROM t"));
}

#[test]
fn floor_char_boundary_and_line_offsets() {
    assert_eq!(floor_char_boundary("ab", 8), 2);
    assert_eq!(floor_char_boundary("ab", 1), 1);
    assert_eq!(floor_char_boundary("á", 1), 0);
    assert_eq!(line_start_offset("a\nb\nc", 2), 2);
    assert_eq!(line_start_offset("a\nb", 9), 3);
    assert_eq!(call_offset("query(`SELECT 1`)", 1), 0);
}
