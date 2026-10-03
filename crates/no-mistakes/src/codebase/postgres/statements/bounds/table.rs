use super::query;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::Table;

/// `TABLE name` is `SELECT * FROM name`: it returns every row of the relation.
pub(super) fn bound(table: &Table, at: (usize, usize)) -> SqlBoundQuery {
    let name: Vec<&str> = [table.schema_name.as_deref(), table.table_name.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if name.is_empty() {
        return query::sized_by_itself(at);
    }
    SqlBoundQuery {
        capped: false,
        items: vec![SqlBoundItem {
            kind: SqlBoundItemKind::Table(name.join(".")),
            alias: None,
            line: at.0,
            column: at.1,
            pins: Vec::new(),
        }],
    }
}
