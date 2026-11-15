use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

fn root() -> std::path::PathBuf {
    super::fixture_root().join("../issue-3452")
}

fn catalog() -> SchemaCatalog {
    let root = root();
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

#[test]
fn partial_index_gaps_follow_the_arm_and_the_column_type() {
    let root = root();
    let catalog = catalog();
    for (name, expected) in [
        ("arm-both.sql", 0),
        ("arm-parameter.sql", 0),
        ("varchar-any.sql", 0),
        ("varchar-any-array.sql", 0),
        ("varchar-in-stored.sql", 0),
        ("text-cast.sql", 0),
        ("arm-null.sql", 1),
        ("arm-null-safe.sql", 1),
        ("citext-text.sql", 1),
        ("citext-literal.sql", 1),
        ("char-text.sql", 1),
        ("char-literal.sql", 1),
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{name}");
        assert_eq!(facts.bounds.len(), 1, "{name}");
        assert_eq!(
            super::super::evaluate::offenders(&facts.bounds[0], &catalog).len(),
            expected,
            "{name}"
        );
    }
}
