use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

fn root() -> std::path::PathBuf {
    super::fixture_root().join("../issue-3394")
}

fn catalog() -> SchemaCatalog {
    let root = root();
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

#[test]
fn partial_unique_indexes_prove_only_implied_predicates() {
    let root = root();
    let catalog = catalog();
    assert_eq!(catalog.bounded_unique_indexes("flows").len(), 1, "flows");
    assert_eq!(
        catalog.bounded_unique_indexes("flows_written").len(),
        1,
        "flows_written"
    );
    assert_eq!(
        catalog.bounded_unique_indexes("bindings").len(),
        2,
        "bindings"
    );
    assert_eq!(
        catalog.bounded_unique_indexes("bindings_text").len(),
        2,
        "bindings_text"
    );
    assert_eq!(
        catalog.unique_keys("bindings"),
        vec![vec!["id".to_string()]]
    );
    assert_eq!(
        catalog
            .enum_type("source_kinds")
            .map(|enum_type| enum_type.values.clone()),
        Some(vec!["direct".to_string(), "family".to_string()])
    );
    assert_eq!(
        catalog
            .enum_type("\"SourceKinds\"")
            .map(|enum_type| enum_type.values.clone()),
        Some(vec!["direct".to_string(), "family".to_string()])
    );
    for (name, expected) in [
        ("flows-in.sql", &[0][..]),
        ("flows-written.sql", &[0]),
        ("reordered-in.sql", &[0]),
        ("flows-any.sql", &[0]),
        ("bindings-direct.sql", &[0]),
        ("bindings-forward.sql", &[0]),
        ("bindings-literal-left.sql", &[0]),
        ("flows-expression-null.sql", &[0]),
        ("flows-join.sql", &[0]),
        ("bindings-or.sql", &[0]),
        ("bindings-enum-both.sql", &[0]),
        ("bindings-enum-join.sql", &[1, 0]),
        ("casts-pass.sql", &[0, 0, 0, 0, 0, 0]),
        ("casts-report.sql", &[1, 1, 1, 1, 1, 1, 1]),
        ("quoted-enum.sql", &[0, 1]),
        ("controls.sql", &[0, 0]),
        ("wider-in.sql", &[1]),
        ("narrower-in.sql", &[1]),
        ("missing-conjunct.sql", &[1]),
        ("parameter-literal.sql", &[1]),
        ("parameter-one-literal.sql", &[1]),
        ("equality-literal.sql", &[1]),
        ("text-or.sql", &[1]),
        ("text-both.sql", &[1]),
        ("enum-missing-user.sql", &[1]),
        ("bindings-conflict.sql", &[1]),
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{name}");
        assert_eq!(facts.bounds.len(), expected.len(), "{name}");
        for (index, fact) in facts.bounds.iter().enumerate() {
            assert_eq!(
                super::super::evaluate::offenders(fact, &catalog).len(),
                expected[index],
                "{name} statement {index}"
            );
        }
    }
}
