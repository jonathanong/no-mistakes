use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

fn fixture(name: &str) -> String {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("sql/cte-returning")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn catalog() -> SchemaCatalog {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements");
    let path = root.join("schema-cte-returning.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    SchemaCatalog::load(&root, "schema-cte-returning.json", &sources).unwrap()
}

fn offenders(name: &str) -> Vec<Vec<String>> {
    let facts = extract_sql_statement_facts(&fixture(name));
    assert!(!facts.parse_failed, "{name}");
    assert_eq!(
        facts.bounds.len(),
        if name == "select-pk.sql" { 1 } else { 2 },
        "{name}"
    );
    let catalog = catalog();
    facts
        .bounds
        .iter()
        .map(|fact| {
            super::super::offenders(fact, &catalog)
                .into_iter()
                .map(|offender| offender.table)
                .collect()
        })
        .collect()
}

#[test]
fn key_bounded_returning_and_select_ctes_bound_outer_primary_key_joins() {
    for name in [
        "update-pk.sql",
        "delete-pk.sql",
        "select-pk.sql",
        "nested-projection.sql",
    ] {
        assert!(offenders(name).iter().all(Vec::is_empty), "{name}");
    }
}

#[test]
fn unbounded_returning_and_nonkey_joins_do_not_gain_a_row_cap() {
    assert_eq!(
        offenders("unbounded-update.sql"),
        [vec!["orders".to_string()], vec!["accounts".to_string()]]
    );
    assert_eq!(
        offenders("auxiliary-unbounded.sql"),
        [vec!["orders".to_string()], vec!["accounts".to_string()]]
    );
    assert_eq!(
        offenders("non-pk-join.sql"),
        [vec![], vec!["accounts".to_string()]]
    );
    assert_eq!(
        offenders("recursive-opaque.sql"),
        [vec![], vec!["accounts".to_string()]]
    );
}
