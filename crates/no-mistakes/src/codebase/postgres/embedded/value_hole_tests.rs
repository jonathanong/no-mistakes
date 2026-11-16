use super::{EmbeddedSqlOptions, TrustedSqlTag, extract_embedded_sql_from_source};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/value-const-holes")
        .join(name)
}

fn sqls(name: &str) -> String {
    let path = fixture(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        }]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    let mut sqls = Vec::new();
    for call in &facts.calls {
        for version in call.statement_calls() {
            sqls.push(version.sql_text.clone().unwrap_or_default());
        }
    }
    sqls.sort();
    sqls.join("\n")
}

#[test]
fn value_constants_stay_bind_parameters() {
    let sql = sqls("src/values.ts");
    assert!(sql.contains("VALUES ($1, $2)"), "{sql}");
    assert!(sql.contains("WHERE slug = sql_placeholder_1"), "{sql}");
    assert!(sql.contains("username = sql_placeholder_1"), "{sql}");
    assert!(sql.contains("LOWER(sql_placeholder_1)"), "{sql}");
    assert!(
        sql.contains("COALESCE(o.storage_key, sql_placeholder_1 || o.id::text)"),
        "{sql}"
    );
    for leaked in ["widget-", "default-widget", "obj-", "keep-", "system"] {
        assert!(!sql.contains(leaked), "{leaked} in {sql}");
    }
}

#[test]
fn adjacent_holes_keep_a_separator() {
    let sql = sqls("src/holes.ts");
    assert!(sql.contains("$2 $3"), "{sql}");
    assert!(sql.contains("WHERE n = $4"), "{sql}");
    assert!(!sql.contains("$2$3"), "{sql}");
    assert!(!sql.contains("$ sql_placeholder"), "{sql}");
    assert!(!sql.contains("$sql_placeholder"), "{sql}");
}

#[test]
fn punctuation_fragments_splice_without_a_separator() {
    let sql = sqls("src/fragments.ts");
    assert!(sql.contains("SELECT $1, other_col FROM widgets"), "{sql}");
    assert!(sql.contains("SELECT $1 FROM widgets"), "{sql}");
    assert!(
        !sql.contains("sql_placeholder_1 sql_placeholder_2"),
        "{sql}"
    );
    assert!(!sql.contains("$1 ,"), "{sql}");
}
