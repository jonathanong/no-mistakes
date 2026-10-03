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
    // The parser keeps no quote information for a TABLE name, so it is read as unquoted.
    let kind = SqlBoundItemKind::Table(name.join(".").to_ascii_lowercase());
    SqlBoundQuery {
        capped: false,
        items: vec![SqlBoundItem::new(kind, None, at)],
    }
}
