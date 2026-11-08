use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn issue_3116_recursive_and_correlated_key_lookups_keep_unsafe_controls() {
    let root = super::fixture_root().join("../issue-3116/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    for (name, expected) in [
        ("recursive.sql", vec![0, 1, 1, 1, 1, 1]),
        ("correlated.sql", vec![0, 1, 0, 1, 1, 1, 1, 0, 1]),
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{name}");
        let counts: Vec<_> = facts
            .bounds
            .iter()
            .map(|fact| super::super::evaluate::offenders(fact, &catalog).len())
            .collect();
        assert_eq!(counts, expected, "{name}");
    }
}
