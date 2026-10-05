use crate::codebase::postgres::{canonical_order_keys, parse_postgres_sql, CanonicalOrderKey};
use anyhow::{bail, Result};
use raw::{raw_conflicts, sanitize};
use single_row::query_is_potentially_multi_row;
use sqlparser::ast::{Insert, OnInsert, Query, SelectItem, SetExpr, Statement, TableObject};
use std::collections::BTreeMap;

mod pinned;
pub use pinned::{expression_is_constant, SqlPinnedRelation};
mod raw;
mod single_row;
#[cfg(test)]
mod single_row_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlConflictInsertFact {
    pub table: String,
    pub target: SqlConflictTarget,
    pub source: SqlInsertSourceShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlConflictTarget {
    Columns {
        expressions: Vec<String>,
        predicate: Option<String>,
    },
    Constraint(String),
    Targetless,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SqlInsertSourceShape {
    pub multi_row: bool,
    /// Set when the source reads one relation pinned by constant equalities; a catalog unique
    /// key among `columns` then proves a single row.
    pub pinned_relation: Option<SqlPinnedRelation>,
    /// Select-list expressions by position, for positional `ORDER BY`; `None` with a wildcard.
    pub select_list: Option<Vec<String>>,
    pub order: Option<Vec<CanonicalOrderKey>>,
    pub projections: Option<BTreeMap<String, String>>,
    pub order_aliases: BTreeMap<String, String>,
}

pub fn analyze_conflict_inserts(sql: &str) -> Result<Vec<SqlConflictInsertFact>> {
    let raw = raw_conflicts(sql)?;
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let sanitized = sanitize(sql, &raw);
    let statements = parse_postgres_sql(&sanitized)?;
    let mut raw = raw.into_iter();
    let mut inserts = Vec::new();
    for statement in &statements {
        collect_statement(statement, &mut raw, &mut inserts)?;
    }
    if raw.next().is_some() {
        bail!("could not align ON CONFLICT clauses with INSERT statements");
    }
    Ok(inserts)
}

fn collect_statement(
    statement: &Statement,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    match statement {
        Statement::Insert(insert) => collect_insert(insert, raw, inserts),
        Statement::Query(query) => collect_query(query, raw, inserts),
        _ => Ok(()),
    }
}

fn collect_query(
    query: &Query,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    let first = inserts.len();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_query(&cte.query, raw, inserts)?;
        }
    }
    collect_set_expr(query.body.as_ref(), raw, inserts)?;
    if let Some(with) = &query.with {
        // A CTE can shadow a catalog table, so its name proves nothing about uniqueness.
        pinned::forget_shadowed(&mut inserts[first..], with);
    }
    Ok(())
}

fn collect_set_expr(
    body: &SetExpr,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    match body {
        SetExpr::Insert(statement) => collect_statement(statement, raw, inserts),
        SetExpr::Query(query) => collect_query(query, raw, inserts),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set_expr(left, raw, inserts)?;
            collect_set_expr(right, raw, inserts)
        }
        _ => Ok(()),
    }
}

fn collect_insert(
    insert: &Insert,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    if let Some(source) = insert.source.as_deref() {
        collect_query(source, raw, inserts)?;
    }
    let Some(OnInsert::OnConflict(_)) = insert.on else {
        return Ok(());
    };
    let raw = raw
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing ON CONFLICT target"))?;
    inserts.push(analyze_insert(insert, raw.target)?);
    Ok(())
}

fn analyze_insert(insert: &Insert, target: SqlConflictTarget) -> Result<SqlConflictInsertFact> {
    let table = match &insert.table {
        TableObject::TableName(name) => name.to_string(),
        _ => bail!("INSERT target is not a table name"),
    };
    let source = insert
        .source
        .as_deref()
        .map_or(SqlInsertSourceShape::default(), |query| {
            SqlInsertSourceShape {
                multi_row: query_is_potentially_multi_row(query),
                pinned_relation: pinned::pinned_relation(query),
                select_list: single_row::select_list(query.body.as_ref()),
                order: query.order_by.as_ref().and_then(canonical_order_keys),
                projections: projection_map(insert, query.body.as_ref()),
                order_aliases: order_aliases(query.body.as_ref()),
            }
        });
    Ok(SqlConflictInsertFact {
        table,
        target,
        source,
    })
}

fn projection_map(insert: &Insert, body: &SetExpr) -> Option<BTreeMap<String, String>> {
    if insert.columns.is_empty() {
        return None;
    }
    let SetExpr::Select(select) = body else {
        return None;
    };
    if insert.columns.len() != select.projection.len() {
        return None;
    }
    insert
        .columns
        .iter()
        .zip(&select.projection)
        .map(|(column, projection)| {
            let expression = match projection {
                SelectItem::UnnamedExpr(expression)
                | SelectItem::ExprWithAlias {
                    expr: expression, ..
                } => expression.to_string(),
                _ => return None,
            };
            Some((column.to_string().to_ascii_lowercase(), expression))
        })
        .collect()
}

fn order_aliases(body: &SetExpr) -> BTreeMap<String, String> {
    let SetExpr::Select(select) = body else {
        return BTreeMap::new();
    };
    select
        .projection
        .iter()
        .filter_map(|projection| {
            let SelectItem::ExprWithAlias { expr, alias } = projection else {
                return None;
            };
            Some((alias.value.to_ascii_lowercase(), expr.to_string()))
        })
        .collect()
}
