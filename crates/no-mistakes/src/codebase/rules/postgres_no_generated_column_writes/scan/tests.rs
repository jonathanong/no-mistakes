use super::*;

#[test]
fn combined_catalog_reports_both_column_kinds() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-generated-column-writes/unit-fixture/both-column-kinds",
    );
    let paths = [root.join("schema.sql"), root.join("writes.sql")];
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let schema =
        crate::codebase::postgres::extract_schema_facts(&root, &sources, &paths[..1]).unwrap();
    let tables = super::super::catalog::live_tables(&schema);
    let generated = super::super::catalog::catalog_from_tables(&tables, &[]);
    let trigger =
        super::super::catalog::trigger_catalog_from_tables(&tables, &["updated_at".into()]);
    let mut combined = trigger.clone();
    combined.extend_from(&generated);
    let findings = scan_sql_file(&paths[1], "writes.sql", &sources, &combined, &generated);
    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("trigger-maintained")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("generated column")));
}
