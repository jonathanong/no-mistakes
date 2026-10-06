use crate::codebase::postgres::{canonical_order_keys, parse_postgres_sql, CanonicalOrderKey};
use anyhow::{bail, Result};
use collect::collect_statement;
use raw::{raw_conflicts, sanitize};
use single_row::query_is_potentially_multi_row;
use sqlparser::ast::{Insert, SelectItem, SetExpr, TableObject};
use std::collections::{BTreeMap, BTreeSet};

mod collect;
mod pinned;
pub use pinned::{expression_is_constant, SqlPinnedRelation};
mod raw;
mod scope;
pub use scope::SqlSourceRelation;
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
    /// Select-list expressions that are literals or bound parameters.
    pub constant_projections: BTreeSet<String>,
    /// The `FROM` relations, or `None` when they cannot be listed.
    pub relations: Option<Vec<SqlSourceRelation>>,
}

pub fn analyze_conflict_inserts(sql: &str) -> Result<Vec<SqlConflictInsertFact>> {
    analyze_conflict_inserts_with_binds(sql, &[])
}

/// `binds` are the `(line, column)` positions of identifiers recovered from template
/// interpolations; only those identifiers count as bound values.
pub fn analyze_conflict_inserts_with_binds(
    sql: &str,
    binds: &[(u32, u32)],
) -> Result<Vec<SqlConflictInsertFact>> {
    let raw = raw_conflicts(sql)?;
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let sanitized = sanitize(sql, &raw);
    let statements = parse_postgres_sql(&sanitized)?;
    let mut raw = raw.into_iter();
    let mut inserts = Vec::new();
    for statement in &statements {
        collect_statement(statement, &mut raw, binds, &mut inserts)?;
    }
    if raw.next().is_some() {
        bail!("could not align ON CONFLICT clauses with INSERT statements");
    }
    Ok(inserts)
}

fn analyze_insert(
    insert: &Insert,
    target: SqlConflictTarget,
    binds: &[(u32, u32)],
) -> Result<SqlConflictInsertFact> {
    let table = match &insert.table {
        TableObject::TableName(name) => name.to_string(),
        _ => bail!("INSERT target is not a table name"),
    };
    let source = insert
        .source
        .as_deref()
        .map_or(SqlInsertSourceShape::default(), |query| {
            SqlInsertSourceShape {
                multi_row: query_is_potentially_multi_row(query, binds),
                pinned_relation: pinned::pinned_relation(query, binds),
                select_list: single_row::select_list(query.body.as_ref()),
                order: query.order_by.as_ref().and_then(canonical_order_keys),
                projections: projection_map(insert, query.body.as_ref()),
                order_aliases: order_aliases(query.body.as_ref()),
                constant_projections: scope::constant_projections(query, binds),
                relations: scope::source_relations(query),
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
