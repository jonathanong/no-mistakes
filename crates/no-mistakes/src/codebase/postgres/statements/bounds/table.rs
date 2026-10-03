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
    // sqlparser loses TABLE identifier quoting. Conservatively retain both spellings
    // when folding changes a name; catalog resolution selects the relation that exists.
    let exact = name
        .iter()
        .map(|part| format!("\"{}\"", part.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(".");
    let folded = name.join(".").to_ascii_lowercase();
    let cte = if table.schema_name.is_none() {
        scope
            .get(table.table_name.as_deref().unwrap_or_default())
            .or_else(|| scope.get(&folded))
    } else {
        None
    };
    let items = match cte {
        Some(bound) => vec![SqlBoundItem::new(
            SqlBoundItemKind::Query(bound.clone()),
            None,
            at,
        )],
        None => {
            let mut items = vec![SqlBoundItem::new(SqlBoundItemKind::Table(folded), None, at)];
            if name
                .iter()
                .any(|part| part.to_ascii_lowercase() != *part || part.contains(' '))
            {
                items.push(SqlBoundItem::new(SqlBoundItemKind::Table(exact), None, at));
            }
            items
        }
    };
    SqlBoundQuery {
        capped: false,
        items,
    }
}
