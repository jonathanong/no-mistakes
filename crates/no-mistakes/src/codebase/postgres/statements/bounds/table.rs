use super::{query, Scope};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::Table;

/// `TABLE name` is `SELECT * FROM name`: it returns every row of the relation.
pub(super) fn bound(table: &Table, scope: &Scope, at: (usize, usize)) -> SqlBoundQuery {
    let name: Vec<&str> = [table.schema_name.as_deref(), table.table_name.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if name.is_empty() {
        return query::sized_by_itself(at);
    }
    // The parser keeps no quote information for a TABLE name, so it is read as unquoted.
    let name = name.join(".").to_ascii_lowercase();
    // A one-part name is a CTE reference when a CTE has that name, like any FROM item.
    let cte = scope.get(&name).filter(|_| table.schema_name.is_none());
    let kind = match cte {
        Some(bound) => SqlBoundItemKind::Query(bound.clone()),
        None => SqlBoundItemKind::Table(name),
    };
    SqlBoundQuery {
        capped: false,
        items: vec![SqlBoundItem::new(kind, None, at)],
    }
}
