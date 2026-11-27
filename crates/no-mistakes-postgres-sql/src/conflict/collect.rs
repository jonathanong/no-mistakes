//! Walk statements, CTEs and set operations to the `INSERT ... ON CONFLICT` facts inside them.
use super::{analyze_insert, pinned, raw, SqlConflictInsertFact};
use anyhow::Result;
use sqlparser::ast::{Insert, OnInsert, Query, SetExpr, Statement};

pub(super) fn collect_statement(
    statement: &Statement,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    binds: &[(u32, u32)],
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    match statement {
        Statement::Insert(insert) => collect_insert(insert, raw, binds, inserts),
        Statement::Query(query) => collect_query(query, raw, binds, inserts),
        _ => Ok(()),
    }
}

fn collect_query(
    query: &Query,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    binds: &[(u32, u32)],
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    let first = inserts.len();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_query(&cte.query, raw, binds, inserts)?;
        }
    }
    collect_set_expr(query.body.as_ref(), raw, binds, inserts)?;
    if let Some(with) = &query.with {
        // A CTE can shadow a catalog table, so its name proves nothing about uniqueness.
        pinned::forget_shadowed(&mut inserts[first..], with);
    }
    Ok(())
}

fn collect_set_expr(
    body: &SetExpr,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    binds: &[(u32, u32)],
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    match body {
        SetExpr::Insert(statement) => collect_statement(statement, raw, binds, inserts),
        SetExpr::Query(query) => collect_query(query, raw, binds, inserts),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set_expr(left, raw, binds, inserts)?;
            collect_set_expr(right, raw, binds, inserts)
        }
        _ => Ok(()),
    }
}

fn collect_insert(
    insert: &Insert,
    raw: &mut std::vec::IntoIter<raw::RawConflict>,
    binds: &[(u32, u32)],
    inserts: &mut Vec<SqlConflictInsertFact>,
) -> Result<()> {
    if let Some(source) = insert.source.as_deref() {
        collect_query(source, raw, binds, inserts)?;
    }
    let Some(OnInsert::OnConflict(_)) = insert.on else {
        return Ok(());
    };
    let raw = raw
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing ON CONFLICT target"))?;
    inserts.push(analyze_insert(insert, raw.target, binds)?);
    Ok(())
}
