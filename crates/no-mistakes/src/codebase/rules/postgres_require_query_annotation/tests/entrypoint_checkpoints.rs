use super::*;

#[test]
fn checkpoint_task_counts_preserve_known_public_findings() {
    let root = fixture("helper-tracing-entrypoint-checkpoints");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let config = config_with_options(
        "importSpecifier: '@app/db'\nexecutorNames: [read, query]\ninclude: ['zero.mts', 'zero-dense.mts', 'one.mts', 'precise-aliases.mts']\nunanalyzableSql: report",
    );
    let findings = check_with_files(&root, &config, &files).unwrap();
    // Frozen old public CLI output: zero, one and multiple speculative tasks.
    // Unknown SQL is deliberately reported, so it cannot replace an exact row.
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        vec![
            ("one.mts", 7),
            ("precise-aliases.mts", 2),
            ("precise-aliases.mts", 4),
            ("zero-dense.mts", 134),
            ("zero.mts", 5)
        ],
        "{findings:#?}",
    );
    for finding in findings {
        assert_eq!(finding.target.as_deref(), Some("annotation"));
        assert!(
            finding
                .message
                .ends_with("query execution must start with a /* name */ annotation"),
            "{finding:?}"
        );
    }
}

#[test]
fn supplied_source_public_records_match_frozen_exact_main() {
    use crate::codebase::postgres::{
        extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
    };
    let root = fixture("helper-tracing-entrypoint-checkpoints");
    let sources = [
        "array-many.mts",
        "array-one.mts",
        "captured.mts",
        "db.mts",
        "many.mts",
        "one.mts",
        "precise-aliases.mts",
        "precise-lookahead.mts",
        "zero-dense.mts",
        "zero.mts",
    ]
    .map(|name| ExecutedPostgresSource {
        file_name: name.into(),
        source: std::fs::read_to_string(root.join(name)).unwrap(),
    });
    let records = extract_executed_postgres_sql_with_imports(
        &sources,
        &EmbeddedSqlOptions::configured("@app/db", &[]),
        &[],
    )
    .unwrap();
    // Saved from the actual d7a905 public API, including conservative records.
    // Full JSON locks shape as well as SQL/annotation/completeness semantics.
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected-precise.json")).unwrap())
            .unwrap();
    assert_eq!(serde_json::to_value(&records).unwrap(), expected);
    for name in ["array-one.mts", "array-many.mts"] {
        assert!(
            records.iter().any(|record| record.file_name == name
                && record.complete
                && record.annotation.is_some()),
            "{records:?}"
        );
        assert!(
            records.iter().any(|record| record.file_name == name
                && record.complete
                && record.annotation.is_none()
                && record.sql_text.as_deref() == Some("SELECT 1")),
            "{records:?}"
        );
    }
}
