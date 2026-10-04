use super::{walk_factor, SqlRelationPredicateFact, SqlSelectFact};
use sqlparser::ast::{Expr, FromTable, TableWithJoins, UpdateTableFromKind};

pub(in crate::codebase::postgres::statements::mutations) fn from_tables(
    from: &UpdateTableFromKind,
) -> &[TableWithJoins] {
    match from {
        UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables) => tables,
    }
}

pub(in crate::codebase::postgres::statements::mutations) fn delete_tables(
    from: &FromTable,
) -> Vec<TableWithJoins> {
    match from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables.clone(),
    }
}

pub(in crate::codebase::postgres::statements::mutations) fn push_group(
    sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<Vec<SqlRelationPredicateFact>>,
) {
    let relations =
        super::super::predicates::write_relations(sql, tables, selection, ctes, positions);
    if !relations.is_empty() {
        out.push(relations);
    }
}

pub(in crate::codebase::postgres::statements::mutations) fn walk_side_queries(
    sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
    positions: super::super::value::PlaceholderPositions<'_>,
) {
    if let Some(selection) = selection {
        super::super::select::collect_predicate_shapes(selection, selects);
        super::factor::collect_exists_facts(sql, selection, positions, selects);
        super::super::select::walk_expr_at(sql, selection, ctes, false, positions, selects);
    }
    for table in tables {
        walk_factor(sql, &table.relation, ctes, selects, positions);
        for join in &table.joins {
            walk_factor(sql, &join.relation, ctes, selects, positions);
            if let Some(expr) = super::super::select::join_expr(&join.join_operator) {
                super::super::select::collect_predicate_shapes(expr, selects);
                super::factor::collect_exists_facts(sql, expr, positions, selects);
                super::super::select::walk_expr_at(sql, expr, ctes, false, positions, selects);
            }
        }
    }
}
