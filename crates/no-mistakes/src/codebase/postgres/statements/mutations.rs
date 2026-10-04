mod conflict;
mod factor;
mod merge;
mod query;
mod returning;
mod side_queries;
use super::{SqlRelationPredicateFact, SqlSelectFact};
use factor::walk_factor;
use query::collect_query;
use returning::walk_returning;
use side_queries::{delete_tables, from_tables, push_group, walk_side_queries};
use sqlparser::ast::Statement;

pub(super) fn collect(
    sql: &str,
    statement: &Statement,
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::SqlColumnUseFact>,
    positions: super::value::PlaceholderPositions<'_>,
) {
    collect_in_scope(
        sql,
        statement,
        &[],
        updates,
        deletes,
        selects,
        column_uses,
        positions,
    );
}

fn collect_in_scope(
    sql: &str,
    statement: &Statement,
    ctes: &[String],
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::SqlColumnUseFact>,
    positions: super::value::PlaceholderPositions<'_>,
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
            walk_side_queries(
                sql,
                &tables,
                update.selection.as_ref(),
                ctes,
                selects,
                positions,
            );
            for assignment in &update.assignments {
                super::select::walk_expr_at(
                    sql,
                    &assignment.value,
                    ctes,
                    false,
                    positions,
                    selects,
                );
            }
            walk_returning(sql, update.returning.as_deref(), ctes, positions, selects);
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
            walk_side_queries(
                sql,
                &tables,
                delete.selection.as_ref(),
                ctes,
                selects,
                positions,
            );
            walk_returning(sql, delete.returning.as_deref(), ctes, positions, selects);
        }
        Statement::Query(query) => collect_query(
            sql,
            query,
            ctes,
            updates,
            deletes,
            selects,
            column_uses,
            positions,
        ),
        Statement::Insert(insert) => {
            if let Some(source) = insert.source.as_deref() {
                collect_query(
                    sql,
                    source,
                    ctes,
                    updates,
                    deletes,
                    selects,
                    column_uses,
                    positions,
                );
            }
            conflict::collect(sql, insert.on.as_ref(), ctes, selects, positions);
            walk_returning(sql, insert.returning.as_deref(), ctes, positions, selects);
        }
        Statement::Merge(statement) => merge::collect(sql, statement, ctes, selects, positions),
        _ => {}
    }
}
