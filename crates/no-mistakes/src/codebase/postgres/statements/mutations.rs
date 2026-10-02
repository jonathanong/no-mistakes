mod returning;
use super::{SqlRelationPredicateFact, SqlSelectFact};
use returning::walk_returning;
use sqlparser::ast::{
    Expr, FromTable, Statement, TableFactor, TableWithJoins, UpdateTableFromKind,
};

pub(super) fn collect(
    sql: &str,
    statement: &Statement,
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::SqlColumnUseFact>,
) {
    collect_in_scope(sql, statement, &[], updates, deletes, selects, column_uses);
}

fn collect_in_scope(
    sql: &str,
    statement: &Statement,
    ctes: &[String],
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::SqlColumnUseFact>,
) {
    match statement {
        Statement::Update(update) => {
            let mut tables = vec![update.table.clone()];
            if let Some(from) = &update.from {
                tables.extend(from_tables(from).iter().cloned());
            }
            column_uses.extend(super::select::mutation_column_uses(
                &tables,
                update.selection.as_ref(),
                ctes,
            ));
            push_group(sql, &tables, update.selection.as_ref(), ctes, updates);
            walk_side_queries(sql, &tables, update.selection.as_ref(), ctes, selects);
            for assignment in &update.assignments {
                super::select::walk_expr(sql, &assignment.value, ctes, false, selects);
            }
            walk_returning(sql, update.returning.as_deref(), ctes, selects);
        }
        Statement::Delete(delete) => {
            let mut tables = delete_tables(&delete.from);
            if let Some(using) = &delete.using {
                tables.extend(using.iter().cloned());
            }
            column_uses.extend(super::select::mutation_column_uses(
                &tables,
                delete.selection.as_ref(),
                ctes,
            ));
            push_group(sql, &tables, delete.selection.as_ref(), ctes, deletes);
            walk_side_queries(sql, &tables, delete.selection.as_ref(), ctes, selects);
            walk_returning(sql, delete.returning.as_deref(), ctes, selects);
        }
        Statement::Query(query) => {
            collect_query(sql, query, ctes, updates, deletes, selects, column_uses)
        }
        Statement::Insert(insert) => {
            if let Some(source) = insert.source.as_deref() {
                collect_query(sql, source, ctes, updates, deletes, selects, column_uses);
            }
            walk_returning(sql, insert.returning.as_deref(), ctes, selects);
        }
        _ => {}
    }
}

fn collect_query(
    sql: &str,
    query: &sqlparser::ast::Query,
    outer: &[String],
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::SqlColumnUseFact>,
) {
    let mut ctes = outer.to_vec();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            let name = crate::codebase::postgres::idents::ident_key(&cte.alias.name);
            if with.recursive {
                ctes.push(name.clone());
            }
            collect_query(
                sql,
                &cte.query,
                &ctes,
                updates,
                deletes,
                selects,
                column_uses,
            );
            if !with.recursive {
                ctes.push(name);
            }
        }
    }
    collect_set(
        sql,
        &query.body,
        &ctes,
        updates,
        deletes,
        selects,
        column_uses,
    );
}

fn collect_set(
    sql: &str,
    set: &sqlparser::ast::SetExpr,
    ctes: &[String],
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::SqlColumnUseFact>,
) {
    use sqlparser::ast::SetExpr;
    match set {
        SetExpr::Update(statement) | SetExpr::Delete(statement) | SetExpr::Insert(statement) => {
            collect_in_scope(sql, statement, ctes, updates, deletes, selects, column_uses)
        }
        SetExpr::Query(query) => {
            collect_query(sql, query, ctes, updates, deletes, selects, column_uses)
        }
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(sql, left, ctes, updates, deletes, selects, column_uses);
            collect_set(sql, right, ctes, updates, deletes, selects, column_uses);
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
    ctes: &[String],
    out: &mut Vec<Vec<SqlRelationPredicateFact>>,
) {
    let relations = super::predicates::write_relations(sql, tables, selection, ctes);
    if !relations.is_empty() {
        out.push(relations);
    }
}

fn walk_side_queries(
    sql: &str,
    tables: &[TableWithJoins],
    selection: Option<&Expr>,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
) {
    if let Some(selection) = selection {
        super::select::walk_expr(sql, selection, ctes, false, selects);
    }
    for table in tables {
        walk_factor(sql, &table.relation, ctes, selects);
        for join in &table.joins {
            walk_factor(sql, &join.relation, ctes, selects);
            if let Some(expr) = super::select::join_expr(&join.join_operator) {
                super::select::walk_expr(sql, expr, ctes, false, selects);
            }
        }
    }
}

fn walk_factor(sql: &str, factor: &TableFactor, ctes: &[String], selects: &mut Vec<SqlSelectFact>) {
    match factor {
        TableFactor::Derived { subquery, .. } => {
            super::select::collect_query(sql, subquery, ctes, false, false, selects);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => walk_side_queries(
            sql,
            std::slice::from_ref(table_with_joins),
            None,
            ctes,
            selects,
        ),
        _ => {}
    }
}
