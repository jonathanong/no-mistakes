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
    let item = |relation: String, cte_name: &str| {
        let cte = table
            .schema_name
            .is_none()
            .then(|| scope.get(cte_name))
            .flatten();
        SqlBoundItem::new(
            match cte {
                Some(bound) => SqlBoundItemKind::Query(bound.clone()),
                None => SqlBoundItemKind::Table(relation),
            },
            None,
            at,
        )
    };
    // A possible CTE spelling resolves only that interpretation; it cannot hide the other one.
    let mut items = vec![item(folded.clone(), &folded)];
    if name
        .iter()
        .any(|part| part.to_ascii_lowercase() != *part || part.contains(' '))
    {
        items.push(item(exact, &name.join(".")));
    }
    SqlBoundQuery {
        capped: false,
        items,
    }
}
