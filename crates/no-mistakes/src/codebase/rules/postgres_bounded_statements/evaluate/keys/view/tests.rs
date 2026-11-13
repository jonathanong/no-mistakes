use super::super::super::Evaluation;
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundPin, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind};
use std::collections::BTreeMap;

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("view-key-evidence/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn evaluation(bounded: bool) -> Evaluation {
    Evaluation {
        bounded,
        items: Vec::new(),
        finite_outputs: None,
        offenders: Vec::new(),
    }
}

fn pin(column: &str, source: SqlPinSource) -> SqlBoundPin {
    SqlBoundPin {
        alternative: Some(0),
        column: column.to_owned(),
        unqualified_candidate: true,
        expression: None,
        source,
        null_safe: false,
        reads: Vec::new(),
        qualified_reads: Vec::new(),
        peer_column: None,
    }
}

fn credits(keys: &[Vec<Vec<Vec<Vec<usize>>>>]) -> bool {
    keys.iter()
        .flatten()
        .flatten()
        .flatten()
        .any(|items| items.is_empty())
}

#[test]
fn region_view_pins_credit_only_a_bounded_visible_key() {
    let found = catalog();
    let mut item = SqlBoundItem::new(SqlBoundItemKind::Other, None, (1, 1));
    item.pin_alternatives = 2;
    item.pins = vec![
        pin("visible_id", SqlPinSource::Query(SqlBoundQuery::default())),
        pin("other", SqlPinSource::Query(SqlBoundQuery::default())),
        pin("visible_id", SqlPinSource::StoredArray(Vec::new())),
        pin(
            "visible_id",
            SqlPinSource::ReadQuery(SqlBoundQuery::default()),
        ),
        pin("visible_id", SqlPinSource::Value),
    ];
    let owners = BTreeMap::from([("visible_id".to_owned(), 0)]);
    let subqueries = [
        Some(evaluation(false)),
        Some(evaluation(true)),
        None,
        None,
        None,
    ];
    let keys = super::keys_for(
        &item,
        0,
        &owners,
        &subqueries,
        &[true, true, true, true, true],
        &found,
        "aliased",
    );
    assert!(credits(&keys));
    let mut missed = item.pins[4].clone();
    missed.column = "visible_id".into();
    item.pins = vec![missed];
    item.pin_alternatives = 1;
    let keys = super::keys_for(
        &item,
        0,
        &BTreeMap::from([("visible_id".to_owned(), 9)]),
        &[None],
        &[true],
        &found,
        "aliased",
    );
    assert!(!credits(&keys));
}
