use super::{query, Scope};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::Table;

mod tokens;
pub(in super::super) use tokens::{TableTokenCursor, TableTokenIndex};

/// `TABLE name` is `SELECT * FROM name`: it returns every row of the relation.
pub(super) fn bound(table: &Table, scope: &Scope, at: (usize, usize)) -> SqlBoundQuery {
    let name: Vec<&str> = [table.schema_name.as_deref(), table.table_name.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if name.is_empty() {
        return query::sized_by_itself(at);
    }
    let item = |relation: String, cte_name: &str, at| {
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
    let recovered = scope
        .table_tokens
        .as_ref()
        .and_then(|cursor| cursor.borrow_mut().take(table));
    let items = if let Some(source) = recovered {
        vec![item(source.name, &source.key, source.at)]
    } else {
        // When source tokens are unavailable, retain both plausible spellings as main did.
        let exact = name
            .iter()
            .map(|part| format!("\"{}\"", part.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(".");
        let folded = name.join(".").to_ascii_lowercase();
        let mut items = vec![item(folded.clone(), &folded, at)];
        if name
            .iter()
            .any(|part| part.to_ascii_lowercase() != *part || part.contains(' '))
        {
            items.push(item(exact, &name.join("."), at));
        }
        items
    };
    SqlBoundQuery {
        capped: false,
        items,
    }
}
