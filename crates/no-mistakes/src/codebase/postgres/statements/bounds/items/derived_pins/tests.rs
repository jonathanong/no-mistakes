use super::{apply, item_at};
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundPin, SqlBoundQuery, SqlPinSource,
};
use std::collections::BTreeMap;

fn pin(column: &str) -> SqlBoundPin {
    SqlBoundPin {
        alternative: None,
        column: column.to_string(),
        unqualified_candidate: false,
        expression: None,
        source: SqlPinSource::Value,
        null_safe: false,
        reads: Vec::new(),
        qualified_reads: Vec::new(),
        peer_column: None,
    }
}

fn table(name: &str) -> SqlBoundItem {
    SqlBoundItem::new(SqlBoundItemKind::Table(name.to_string()), None, (1, 1))
}

#[test]
fn a_path_through_a_base_table_does_not_invent_an_inner_item() {
    let mut query = SqlBoundQuery {
        items: vec![table("public.accounts")],
        ..SqlBoundQuery::default()
    };
    assert!(item_at(&mut query, &[0, 0]).is_none());
}

#[test]
fn apply_skips_a_path_that_does_not_resolve() {
    let mut query = SqlBoundQuery {
        items: vec![table("public.accounts")],
        ..SqlBoundQuery::default()
    };
    let mut planned = BTreeMap::new();
    planned.insert(vec![0, 0], vec![("id".to_string(), pin("id"))]);
    apply(&mut query, 1, &planned);
    assert!(query.items[0].pins.is_empty());
}
