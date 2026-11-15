use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

fn catalog() -> SchemaCatalog {
    let root = super::fixture_root().join("../issue-3436/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

#[test]
fn or_arms_sum_partial_unique_keys_and_reject_unbounded_arms() {
    let root = super::fixture_root().join("../issue-3436/fixture");
    let catalog = catalog();
    for (name, count) in [
        ("pass.sql", 0),
        ("like-arm.sql", 1),
        ("unlimited-in.sql", 1),
        ("missing-revoked.sql", 1),
        ("null-safe-arm.sql", 1),
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{name}");
        assert_eq!(facts.bounds.len(), 1, "{name}");
        assert_eq!(
            super::super::evaluate::offenders(&facts.bounds[0], &catalog).len(),
            count,
            "{name}"
        );
    }
}
