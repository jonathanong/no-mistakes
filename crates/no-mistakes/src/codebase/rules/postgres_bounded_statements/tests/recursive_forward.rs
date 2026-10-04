use super::names;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn recursive_forward_ctes_report_the_unbounded_catalog_leaf() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/recursive-forward-bounds.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts"]);
}

#[test]
fn recursive_table_arms_report_exact_catalog_leaves() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    let catalog_path = root.join("schema-recursive-table.json");
    let sources =
        crate::codebase::rules::source_store_for_files(std::slice::from_ref(&catalog_path));
    let catalog = SchemaCatalog::load(&root, "schema-recursive-table.json", &sources).unwrap();
    for (fixture, expected) in [
        ("recursive-table-forward.sql", vec!["accounts"]),
        (
            "recursive-table-quoted-union.sql",
            vec!["\"billing.accounts\"", "accounts"],
        ),
        (
            "recursive-table-qualified-union.sql",
            vec!["billing.accounts", "accounts"],
        ),
    ] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{fixture} must parse as one statement");
        assert_eq!(facts.bounds.len(), 1, "{fixture}");
        let found = facts
            .bounds
            .iter()
            .flat_map(|fact| super::offenders(fact, &catalog))
            .map(|offender| offender.table)
            .collect::<Vec<_>>();
        assert_eq!(found, expected, "{fixture}");
    }
}
