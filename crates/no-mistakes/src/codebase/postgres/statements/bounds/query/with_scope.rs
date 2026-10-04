//! CTE projection under the names and column identities visible to a query.
use super::super::{items, start, Scope};
use super::compact::{compact, size, MAX_BOUND_ITEMS};
use super::{bound_query, modifying_statement, recursive_order};
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::SqlBoundQuery;
use sqlparser::ast::{Query, Spanned};

/// The scope after this query's own `WITH` clause.
pub(in crate::codebase::postgres::statements::bounds) fn with_scope(
    query: &Query,
    outer: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Scope {
    let mut scope = outer.clone();
    let Some(with) = &query.with else {
        return scope;
    };
    if with.recursive {
        // A recursive WITH exposes every alias, including later declarations.
        // Until each body is projected, an unresolved reference remains opaque.
        for cte in &with.cte_tables {
            scope.insert(ident_key(&cte.alias.name), opaque(start(cte.query.span())));
        }
    }
    let order = if with.recursive {
        recursive_order::indices(&with.cte_tables)
    } else {
        (0..with.cte_tables.len()).collect()
    };
    for index in order {
        let cte = &with.cte_tables[index];
        let name = ident_key(&cte.alias.name);
        let bound = if modifying_statement(&cte.query).is_some() {
            // `RETURNING` yields one row per modified row, which nothing in the text sizes: it
            // bounds nothing pinned to it. The statement inside is judged on its own.
            opaque(start(cte.query.span()))
        } else {
            let mut inner = scope.clone();
            if with.recursive {
                // A recursive reference is whatever the recursion has produced so far: it
                // bounds nothing joined to it.
                inner.insert(name.clone(), opaque(start(cte.query.span())));
            }
            let bound = bound_query(&cte.query, &inner, positions);
            // Each reference clones the CTE's bound, so a chain of CTEs that each read the one
            // before twice grows exponentially. Compact oversized bounds conservatively,
            // retaining the distinct uncapped relations instead of hiding their reads.
            if size(&bound) > MAX_BOUND_ITEMS {
                compact(&bound, start(cte.query.span()))
            } else {
                bound
            }
        };
        let columns = if cte.alias.columns.is_empty() {
            super::super::pins::projection_columns(&cte.query)
        } else {
            Some(
                cte.alias
                    .columns
                    .iter()
                    .map(|column| ident_key(&column.name))
                    .collect(),
            )
        };
        scope.columns.insert(name.clone(), columns);
        scope.insert(name, bound);
    }
    scope
}

/// A body whose rows nothing proves bounded, and which is never reported either.
fn opaque(at: (usize, usize)) -> SqlBoundQuery {
    SqlBoundQuery {
        capped: false,
        items: vec![items::opaque(at)],
    }
}
