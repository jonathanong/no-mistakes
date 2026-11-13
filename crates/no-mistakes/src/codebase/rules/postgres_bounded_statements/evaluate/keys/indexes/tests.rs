use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundPin, SqlPinSource};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind};
use sqlparser::ast::Expr;
use std::collections::BTreeMap;

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("unique-index-shapes/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn pin(column: &str, expression: Expr, alternative: Option<usize>) -> SqlBoundPin {
    SqlBoundPin {
        alternative,
        column: column.to_owned(),
        unqualified_candidate: true,
        expression: Some(expression),
        source: SqlPinSource::Value,
        null_safe: false,
        reads: Vec::new(),
        qualified_reads: Vec::new(),
        peer_column: None,
    }
}

fn credits(branch: &[Vec<Vec<Vec<usize>>>]) -> bool {
    branch
        .iter()
        .flatten()
        .flatten()
        .any(|items| items.is_empty())
}

#[test]
fn region_unqualified_index_pins_follow_owner_and_alternative() {
    let found = catalog();
    let table = found.bounded_relation("normalized_emails").unwrap();
    let expression = found
        .bounded_unique_indexes("normalized_emails")
        .iter()
        .flat_map(|index| index.keys.iter())
        .find(|key| !matches!(unwrap_expr(key), Expr::Identifier(_)))
        .unwrap()
        .clone();
    let mut item = SqlBoundItem::new(
        SqlBoundItemKind::Table("normalized_emails".into()),
        None,
        (1, 1),
    );
    item.pins = vec![
        pin("email", expression.clone(), Some(0)),
        pin("other", expression, None),
    ];
    let owners = BTreeMap::from([("email".to_owned(), 0)]);
    let sources = [Some(Vec::new()), Some(Vec::new())];
    let mut prepared = vec![vec![], vec![]];
    super::append(
        &mut prepared,
        &item,
        0,
        &owners,
        ("normalized_emails", table),
        &sources,
        &found,
    );
    assert!(credits(&prepared[0]));
    assert!(!credits(&prepared[1]));
}
