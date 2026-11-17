use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag};
use std::path::PathBuf;

fn case(rule: &str, name: &str, file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join(name)
        .join(file)
}

fn trusted() -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[TrustedSqlTag {
        module: "@example/db".into(),
        name: "sql".into(),
    }])
}

fn sqls(path: PathBuf, options: &EmbeddedSqlOptions) -> Vec<String> {
    let source = std::fs::read_to_string(&path).expect("fixture");
    let facts = extract_embedded_sql_from_source(&path, &source, options);
    let mut sqls = Vec::new();
    for call in &facts.calls {
        for version in call.statement_calls() {
            sqls.push(version.sql_text.clone().unwrap_or_default());
        }
    }
    sqls.sort();
    sqls
}

fn assert_parses(sql: &str) {
    crate::codebase::postgres::parse_postgres_sql(sql).unwrap_or_else(|error| {
        panic!("unparseable: {error}\n{sql}");
    });
}

#[test]
fn dollar_hole_stays_a_positional_parameter() {
    let sqls = sqls(
        case(
            "postgres-lock-ordering",
            "positional-parameters",
            "src/query.ts",
        ),
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    let joined = sqls.join("\n");
    assert!(joined.contains("($1::timestamptz, $2::uuid)"), "{joined}");
    assert!(joined.contains("LIMIT $3 FOR UPDATE"), "{joined}");
    assert!(joined.contains("WHERE id = $1 FOR UPDATE"), "{joined}");
    assert!(!joined.contains("sql_placeholder"), "{joined}");
    for sql in &sqls {
        assert_parses(sql);
    }
}

#[test]
fn append_chains_stay_parseable() {
    let sqls = sqls(
        case("postgres-lock-ordering", "append-chains", "src/query.ts"),
        &trusted(),
    );
    let joined = sqls.join("\n");
    assert!(joined.contains("FOR UPDATE OF w SKIP LOCKED"), "{joined}");
    assert!(joined.contains("FOR UPDATE OF x"), "{joined}");
    assert!(
        joined.contains("WHERE sql_placeholder_1 = sql_placeholder_2"),
        "{joined}"
    );
    assert!(!joined.contains("$ sql_placeholder"), "{joined}");
    for sql in &sqls {
        assert_parses(sql);
    }
}

#[test]
fn numeric_const_limit_still_reaches_conflict_ordering() {
    let sqls = sqls(
        case(
            "postgres-conflict-ordering",
            "numeric-const-limit",
            "src/insert.ts",
        ),
        &trusted(),
    );
    assert_eq!(sqls.len(), 1, "{sqls:?}");
    let sql = &sqls[0];
    assert!(sql.contains("LIMIT sql_placeholder_4 ON CONFLICT"), "{sql}");
    assert!(!sql.contains("LIMIT 100"), "{sql}");
    assert_parses(sql);
    let inserts = crate::codebase::postgres::analyze_conflict_inserts(sql).expect("conflict");
    assert!(inserts[0].source.multi_row, "{inserts:?}");
}
