use super::{SqlRelationPredicateFact, SqlSelectFact};
use sqlparser::ast::{
    Expr, FromTable, Statement, TableFactor, TableWithJoins, UpdateTableFromKind,
};

pub(super) fn collect(
    sql: &str,
    statement: &Statement,
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
) {
    match statement {
        Statement::Update(update) => {
            let mut tables = vec![update.table.clone()];
            if let Some(from) = &update.from {
                tables.extend(from_tables(from).iter().cloned());
            }
            push_group(sql, &tables, update.selection.as_ref(), updates);
            walk_side_queries(sql, &tables, update.selection.as_ref(), selects);
            for assignment in &update.assignments {
                super::select::walk_expr(sql, &assignment.value, &[], false, selects);
            }
        }
        Statement::Delete(delete) => {
            let mut tables = delete_tables(&delete.from);
            if let Some(using) = &delete.using {
                tables.extend(using.iter().cloned());
            }
            push_group(sql, &tables, delete.selection.as_ref(), deletes);
            walk_side_queries(sql, &tables, delete.selection.as_ref(), selects);
        }
        _ => {}
    }
}

fn from_tables(from: &UpdateTableFromKind) -> &[TableWithJoins] {
    match from {
        UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables) => tables,
    }
}

fn delete_tables(from: &FromTable) -> Vec<TableWithJoins> {
    match from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => tables.clone(),
    }
}

fn push_group(
    sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    out: &mut Vec<Vec<SqlRelationPredicateFact>>,
) {
    let relations = super::predicates::write_relations(sql, tables, selection, &[]);
    if !relations.is_empty() {
        out.push(relations);
    }
}

fn walk_side_queries(
    sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    selects: &mut Vec<SqlSelectFact>,
) {
    if let Some(selection) = selection {
        super::select::walk_expr(sql, selection, &[], false, selects);
    }
    for table in tables {
        walk_factor(sql, &table.relation, selects);
        for join in &table.joins {
            walk_factor(sql, &join.relation, selects);
            if let Some(expr) = super::select::join_expr(&join.join_operator) {
                super::select::walk_expr(sql, expr, &[], false, selects);
            }
        }
    }
}

fn walk_factor(sql: &str, factor: &TableFactor, selects: &mut Vec<SqlSelectFact>) {
    match factor {
        TableFactor::Derived { subquery, .. } => {
            super::select::collect_query(sql, subquery, &[], false, false, selects);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => walk_side_queries(sql, std::slice::from_ref(table_with_joins), None, selects),
        _ => {}
    }
}
