use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use crate::codebase::postgres::SchemaCatalog;

#[test]
fn values_and_opaque_items_are_not_unknown_relations() {
    let query = SqlBoundQuery {
        items: vec![
            SqlBoundItem::new(SqlBoundItemKind::Other, None, (1, 1)),
            SqlBoundItem::new(SqlBoundItemKind::Opaque, None, (1, 8)),
        ],
        ..Default::default()
    };
    assert!(!super::has_unknown_relation(
        &query,
        &SchemaCatalog::default(),
        &super::ViewBounds::default()
    ));
}
