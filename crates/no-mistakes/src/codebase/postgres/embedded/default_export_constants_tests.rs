use super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::PathBuf;

fn variant_source(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-required-predicates/variants/default-export-const-sql/src")
        .join(name)
}

fn constants(name: &str) -> Vec<(String, String, bool)> {
    let path = variant_source(name);
    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
    let facts = extract_embedded_sql_from_source(&path, &source, &EmbeddedSqlOptions::default());
    let mut rows: Vec<_> = facts
        .exported_constants
        .iter()
        .map(|(key, constant)| (key.clone(), constant.sql.sql.clone(), constant.fragment))
        .collect();
    rows.sort();
    rows
}

#[test]
fn expression_defaults_use_the_default_local_name() {
    let orders = vec![("default".to_string(), "orders".to_string(), false)];
    for name in [
        "string-table.mts",
        "template-table.mts",
        "paren-table.mts",
        "as-const-table.mts",
        "paren-ident-table.mts",
    ] {
        assert_eq!(constants(name), orders, "{name}");
    }
    let named = constants("named-and-default.mts");
    assert_eq!(
        named,
        vec![
            ("NAMED".to_string(), "named".to_string(), false),
            ("default".to_string(), "orders".to_string(), false),
        ]
    );
}

#[test]
fn an_identifier_default_keeps_the_const_name() {
    assert_eq!(
        constants("ident-table.mts"),
        vec![("NAME".to_string(), "orders".to_string(), false)]
    );
}

#[test]
fn non_literal_defaults_are_not_constants() {
    for name in [
        "substituted.mts",
        "function.mts",
        "arrow.mts",
        "tagged.mts",
        "null.mts",
        "sentinel.mts",
        "let-table.mts",
    ] {
        assert!(constants(name).is_empty(), "{name}");
    }
}
