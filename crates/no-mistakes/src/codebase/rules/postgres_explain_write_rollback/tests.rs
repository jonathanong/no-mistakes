use super::*;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-explain-write-rollback"),
    )
}

#[test]
fn explain_write_rule_direct_entrypoint_respects_sql_scope() {
    let root = fixture().join("fixture/cases");
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .unwrap();
    let plans = root.join("benchmarks/transactions.sql");
    let findings = check_with_files(
        &root,
        &config,
        &[plans, root.join("other/outside.sql"), root.join(".no-mistakes.yml")],
    )
    .unwrap();
    assert_eq!(findings.len(), 8);
    assert!(findings
        .iter()
        .all(|finding| finding.file == "benchmarks/transactions.sql"));
    assert_eq!(findings[3].target.as_deref(), Some("MERGE"));
}

#[test]
fn explain_write_rule_rejects_invalid_sql_scope_options() {
    let fixture = fixture();
    let root = fixture.join("fixture/example");
    let sql_file = root.join("benchmarks/update.sql");
    for name in ["invalid-glob.yml", "invalid-scope-type.yml"] {
        let yaml = std::fs::read_to_string(fixture.join("configs").join(name)).unwrap();
        let config = serde_yaml::from_str(&yaml).unwrap();
        let error = check_with_files(&root, &config, std::slice::from_ref(&sql_file)).unwrap_err();
        assert!(error.to_string().contains("sqlInclude"), "{name}: {error}");
    }
}

#[test]
fn explain_write_rule_keeps_source_read_errors_in_scope() {
    let root = fixture().join("fixture/example");
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .unwrap();
    let outside = root.join("other/absent.sql");
    assert!(check_with_files(&root, &config, &[outside]).unwrap().is_empty());
    let inside = root.join("benchmarks/absent.sql");
    let error = check_with_files(&root, &config, &[inside]).unwrap_err();
    assert!(error.to_string().contains("failed to collect PostgreSQL facts"), "{error}");
}
