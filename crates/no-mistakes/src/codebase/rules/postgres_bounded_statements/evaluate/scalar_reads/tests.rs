use crate::codebase::postgres::statements::{SqlBareRead, SqlBoundItem, SqlBoundQuery};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind, SqlScalarReads};
use std::borrow::Cow;

fn catalog() -> SchemaCatalog {
    let path =
        crate::test_support::rule_fixture_root("postgres-bounded-statements").join("schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn region_scalar_reads_skip_unknown_items_without_adding_pins() {
    let found = catalog();
    let scalar = SqlScalarReads {
        reads: vec![SqlBareRead {
            column: "missing".into(),
            tables: vec!["orders".into()],
        }],
        outer_tables: Vec::new(),
        outer_columns: vec!["missing".into()],
    };
    let query = SqlBoundQuery {
        items: vec![
            SqlBoundItem::new(SqlBoundItemKind::Table("orders".into()), None, (1, 1)),
            SqlBoundItem::new(SqlBoundItemKind::Table("other".into()), None, (1, 1)),
            SqlBoundItem::new(SqlBoundItemKind::Opaque, Some("side".into()), (1, 1)),
        ],
        ..SqlBoundQuery::default()
    };
    let projected = super::project(&query, Some(&scalar), &found);
    assert!(matches!(projected, Cow::Borrowed(_)));
    assert!(projected.items.iter().all(|item| item.pins.is_empty()));
}
