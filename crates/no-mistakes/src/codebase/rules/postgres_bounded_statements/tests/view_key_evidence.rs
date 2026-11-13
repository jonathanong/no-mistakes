use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use crate::codebase::rules::postgres_bounded_statements::evaluate::offenders;

fn root() -> std::path::PathBuf {
    super::fixture_root().join("view-key-evidence")
}

fn catalog() -> SchemaCatalog {
    SchemaCatalog::from_json(&std::fs::read_to_string(root().join("schema.json")).unwrap()).unwrap()
}

#[test]
fn view_lookup_bounds_require_every_key_column_and_bounded_pin_sources() {
    let catalog = catalog();
    let mut mismatches = Vec::new();
    for (name, expected) in [
        ("null_arm", vec![]),
        ("null_or", vec![]),
        ("null_union", vec![]),
        ("null_union_or", vec![]),
        ("null_nested", vec!["base"]),
        ("null_incomplete_or", vec!["base"]),
        ("null_unpinned", vec!["base"]),
        ("null_nonnull_arm", vec!["base"]),
        ("value", vec![]),
        ("array", vec![]),
        ("subquery", vec![]),
        ("array_query", vec![]),
        ("or_complete", vec![]),
        ("item_join", vec![]),
        ("open", vec!["aliased"]),
        ("or_incomplete", vec!["composite"]),
        ("renamed_columns", vec!["aliased"]),
        ("expression", vec!["aliased"]),
        ("outer_read", vec!["aliased"]),
        ("array_read", vec!["aliased"]),
        ("open_subquery", vec!["base"]),
        ("open_array_query", vec!["aliased", "base"]),
        ("open_item_join", vec!["aliased", "base"]),
    ] {
        let sql = std::fs::read_to_string(root().join(format!("{name}.sql"))).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{name}: {facts:?}");
        assert!(!facts.bounds.is_empty(), "{name}: missing statement");
        let mut actual: Vec<_> = facts
            .bounds
            .iter()
            .flat_map(|fact| offenders(fact, &catalog))
            .map(|hit| hit.table)
            .collect();
        actual.sort();
        actual.dedup();
        if actual != expected {
            mismatches.push(format!("{name}: got {actual:?}, expected {expected:?}"));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
