use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/value-const-holes")
        .join(name)
}

fn sqls(path: PathBuf) -> String {
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
    let sql = sqls(fixture("src/values.ts"));
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
    let sql = sqls(fixture("src/holes.ts"));
    assert!(sql.contains("$2 $3"), "{sql}");
    assert!(sql.contains("WHERE n = $4"), "{sql}");
    assert!(!sql.contains("$2$3"), "{sql}");
    assert!(!sql.contains("$ sql_placeholder"), "{sql}");
    assert!(!sql.contains("$sql_placeholder"), "{sql}");
}

#[test]
fn punctuation_fragments_splice_without_a_separator() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/punctuation-fragments.ts");
    let sql = sqls(path);
    assert!(sql.contains("SELECT $1, other_col FROM widgets"), "{sql}");
    assert!(sql.contains("SELECT $1 FROM widgets"), "{sql}");
    assert!(
        !sql.contains("sql_placeholder_1 sql_placeholder_2"),
        "{sql}"
    );
    assert!(!sql.contains("$1 ,"), "{sql}");
}

#[test]
fn raw_tag_binds_text_in_value_expressions() {
    let sql = sqls(fixture("src/value-positions.ts"));
    for expected in [
        "slug LIKE sql_placeholder_1 AND name ILIKE sql_placeholder_2",
        "slug BETWEEN sql_placeholder_1 AND sql_placeholder_2",
        "name || sql_placeholder_1 FROM widgets",
        "INSERT INTO widgets (slug, name) VALUES (sql_placeholder_1, sql_placeholder_2), (sql_placeholder_3, sql_placeholder_4)",
        "(slug, name) > (sql_placeholder_1, sql_placeholder_2)",
        "INSERT INTO widgets (slug, name) VALUES ((sql_placeholder_1), (sql_placeholder_2))",
        "(slug, name) > ((sql_placeholder_1), (sql_placeholder_2))",
        "score + sql_placeholder_1 FROM widgets LIMIT sql_placeholder_2 OFFSET sql_placeholder_3",
        "SELECT name FROM widgets AS w ORDER BY slug, w.name",
        "slug BETWEEN 'a' AND 'z' AND name IS NOT NULL",
        "SELECT $1 FROM widgets",
        "SELECT id FROM widgets WHERE id = (SELECT id FROM widgets LIMIT 1)",
    ] {
        assert!(sql.contains(expected), "missing {expected} in {sql}");
    }
}

#[test]
fn value_positions_respect_sql_token_boundaries_and_nesting() {
    let fixture = super::tests::read_fixture("value-position-contexts.json");
    let cases: Vec<(String, bool)> = serde_json::from_str(&fixture).expect("fixture cases");
    assert!(!cases.is_empty());
    for (prefix, expected) in cases {
        assert_eq!(
            super::helpers::sentinel::value_position(&prefix),
            expected,
            "SQL prefix {prefix:?}",
        );
    }
}
