use super::ViewBounds;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundQuery};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind};
use sqlparser::ast::Expr;
use std::borrow::Cow;

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("unique-index-shapes/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn region_expression_unique_indexes_do_not_project_a_column_pin() {
    let found = catalog();
    assert!(found.bounded_relation("normalized_emails").is_some());
    assert!(found
        .bounded_unique_indexes("normalized_emails")
        .iter()
        .flat_map(|index| &index.keys)
        .any(|key| !matches!(unwrap_expr(key), Expr::Identifier(_))));
    let query = SqlBoundQuery {
        items: vec![SqlBoundItem::new(
            SqlBoundItemKind::Table("normalized_emails".into()),
            None,
            (1, 1),
        )],
        ..SqlBoundQuery::default()
    };
    let projected = super::project(&query, &[], &found, &ViewBounds::default());
    assert!(matches!(projected, Cow::Borrowed(_)));
}
