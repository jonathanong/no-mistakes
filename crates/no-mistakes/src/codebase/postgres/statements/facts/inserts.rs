use super::super::value::PlaceholderPositions;
use super::{insert, SqlInsertFact};
use sqlparser::ast::{Query, SetExpr, Statement};
#[cfg(test)]
mod tests;

pub(super) fn collect_query_inserts(
    sql: &str,
    query: &Query,
    insert_n: &mut usize,
    inserts: &mut Vec<SqlInsertFact>,
    placeholder_positions: PlaceholderPositions<'_>,
) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_query_inserts(sql, &cte.query, insert_n, inserts, placeholder_positions);
        }
    }
    collect_set_inserts(sql, &query.body, insert_n, inserts, placeholder_positions);
}

fn collect_set_inserts(
    sql: &str,
    expr: &SetExpr,
    insert_n: &mut usize,
    inserts: &mut Vec<SqlInsertFact>,
    placeholder_positions: PlaceholderPositions<'_>,
) {
    match expr {
        SetExpr::Insert(statement) => {
            if matches!(statement, Statement::Insert(_)) {
                *insert_n += 1;
                if let Some(fact) =
                    insert::from_statement_at(sql, statement, *insert_n, placeholder_positions)
                {
                    inserts.push(fact);
                }
            }
        }
        SetExpr::Query(query) => {
            collect_query_inserts(sql, query, insert_n, inserts, placeholder_positions)
        }
        SetExpr::SetOperation { left, right, .. } => {
            collect_set_inserts(sql, left, insert_n, inserts, placeholder_positions);
            collect_set_inserts(sql, right, insert_n, inserts, placeholder_positions);
        }
        _ => {}
    }
}
