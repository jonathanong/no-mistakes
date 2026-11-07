use super::tests::fixture;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

fn catalog(name: &str) -> SchemaCatalog {
    let root = fixture();
    let sources = super::super::source_store_for_files(&[root.join(name)]);
    SchemaCatalog::load(&root, name, &sources).unwrap()
}

#[test]
fn joined_views_preserve_probe_direction_and_mirrored_transforms() {
    let catalog = catalog("joined-view-schema.json");
    let columns = super::evaluate::Columns::new(&catalog);
    for status in ["pass", "positive"] {
        let source =
            std::fs::read_to_string(fixture().join(format!("joined-view-{status}.sql"))).unwrap();
        let facts = extract_sql_statement_facts(&source);
        assert!(!facts.parse_failed);
        assert_eq!(facts.transformed_column_uses.len(), 6);
        for use_ in &facts.transformed_column_uses {
            let (table, report) = columns.resolve(use_).unwrap();
            assert_eq!(report, status == "positive", "{status}: {use_:?}");
            if report {
                assert!(matches!(table, "plain_view" | "lateral_view" | "view_join"));
            } else {
                assert_eq!(table, "a");
            }
        }
    }
}

#[test]
fn joined_view_projection_ownership_remains_unambiguous() {
    let catalog = catalog("joined-view-ownership-schema.json");
    let columns = super::evaluate::Columns::new(&catalog);
    let source = std::fs::read_to_string(fixture().join("joined-view-ownership.sql")).unwrap();
    let facts = extract_sql_statement_facts(&source);
    assert!(!facts.parse_failed);
    assert_eq!(facts.transformed_column_uses.len(), 36);
    for use_ in &facts.transformed_column_uses {
        assert_eq!(
            columns.resolve(use_).unwrap().1,
            use_.line > 7,
            "projection on line {}: {use_:?}",
            use_.line,
        );
    }
}
