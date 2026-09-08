use crate::codebase::postgres::{parse_postgres_sql, CanonicalOrderKey};
use anyhow::{bail, Result};
use raw::{raw_conflicts, sanitize};
use sqlparser::ast::{Insert, OnInsert, OrderByKind, SelectItem, SetExpr, Statement, TableObject};
use std::collections::BTreeMap;

mod raw;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlInsertSourceShape {
    pub multi_row: bool,
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
    for statement in statements {
        let Statement::Insert(insert) = statement else {
            continue;
        };
        let Some(OnInsert::OnConflict(_)) = insert.on else {
            continue;
        };
        let raw = raw
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing ON CONFLICT target"))?;
        inserts.push(analyze_insert(&insert, raw.target)?);
    }
    if raw.next().is_some() {
        bail!("could not align ON CONFLICT clauses with INSERT statements");
    }
    Ok(inserts)
}

fn analyze_insert(insert: &Insert, target: SqlConflictTarget) -> Result<SqlConflictInsertFact> {
    let table = match &insert.table {
        TableObject::TableName(name) => name.to_string(),
        _ => bail!("INSERT target is not a table name"),
    };
    let source = insert.source.as_deref().map_or(
        SqlInsertSourceShape {
            multi_row: false,
            order: None,
            projections: None,
            order_aliases: BTreeMap::new(),
        },
        |query| SqlInsertSourceShape {
            multi_row: query_is_potentially_multi_row(query.body.as_ref()),
            order: query.order_by.as_ref().and_then(order_keys),
            projections: projection_map(insert, query.body.as_ref()),
            order_aliases: order_aliases(query.body.as_ref()),
        },
    );
    Ok(SqlConflictInsertFact {
        table,
        target,
        source,
    })
}

fn query_is_potentially_multi_row(body: &SetExpr) -> bool {
    match body {
        SetExpr::Values(values) => values.rows.len() > 1,
        SetExpr::Insert(_) | SetExpr::Update(_) | SetExpr::Delete(_) | SetExpr::Merge(_) => true,
        SetExpr::Query(query) => query_is_potentially_multi_row(query.body.as_ref()),
        SetExpr::Select(_) | SetExpr::SetOperation { .. } | SetExpr::Table(_) => true,
    }
}

fn order_keys(order: &sqlparser::ast::OrderBy) -> Option<Vec<CanonicalOrderKey>> {
    let OrderByKind::Expressions(expressions) = &order.kind else {
        return None;
    };
    Some(
        expressions
            .iter()
            .map(|expression| CanonicalOrderKey {
                expression: expression.expr.to_string(),
                ascending: expression.options.asc.unwrap_or(true),
                nulls_first: expression
                    .options
                    .nulls_first
                    .unwrap_or(!expression.options.asc.unwrap_or(true)),
            })
            .collect(),
    )
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
