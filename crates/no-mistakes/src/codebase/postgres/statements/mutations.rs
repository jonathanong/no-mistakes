mod conflict;
mod factor;
mod merge;
mod query;
mod returning;
mod side_queries;
use super::{SqlRelationPredicateFact, SqlSelectFact};
use factor::walk_factor;
use returning::walk_returning;
use side_queries::{delete_tables, from_tables, push_group, walk_side_queries};
use sqlparser::ast::Statement;

type PlaceholderPositions<'a> = super::value::PlaceholderPositions<'a>;

struct Collector<'sql, 'positions, 'out> {
    sql: &'sql str,
    updates: &'out mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &'out mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &'out mut Vec<SqlSelectFact>,
    column_uses: &'out mut Vec<super::SqlColumnUseFact>,
    positions: PlaceholderPositions<'positions>,
}

pub(super) fn collect(
    sql: &str,
    statement: &Statement,
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::SqlColumnUseFact>,
    positions: PlaceholderPositions<'_>,
) {
    Collector {
        sql,
        updates,
        deletes,
        selects,
        column_uses,
        positions,
    }
    .collect_in_scope(statement, &[]);
}

impl Collector<'_, '_, '_> {
    pub(super) fn collect_in_scope(&mut self, statement: &Statement, ctes: &[String]) {
        match statement {
            Statement::Update(update) => {
                let mut tables = vec![update.table.clone()];
                if let Some(from) = &update.from {
                    tables.extend(from_tables(from).iter().cloned());
                }
                self.column_uses.extend(super::select::mutation_column_uses(
                    &tables,
                    update.selection.as_ref(),
                    ctes,
                ));
                push_group(
                    self.sql,
                    &tables,
                    update.selection.as_ref(),
                    ctes,
                    self.positions,
                    self.updates,
                );
                walk_side_queries(
                    self.sql,
                    &tables,
                    update.selection.as_ref(),
                    ctes,
                    self.selects,
                    self.positions,
                );
                for assignment in &update.assignments {
                    super::select::walk_expr_at(
                        self.sql,
                        &assignment.value,
                        ctes,
                        false,
                        self.positions,
                        self.selects,
                    );
                }
                walk_returning(
                    self.sql,
                    update.returning.as_deref(),
                    ctes,
                    self.positions,
                    self.selects,
                );
            }
            Statement::Delete(delete) => {
                let mut tables = delete_tables(&delete.from);
                if let Some(using) = &delete.using {
                    tables.extend(using.iter().cloned());
                }
                self.column_uses.extend(super::select::mutation_column_uses(
                    &tables,
                    delete.selection.as_ref(),
                    ctes,
                ));
                push_group(
                    self.sql,
                    &tables,
                    delete.selection.as_ref(),
                    ctes,
                    self.positions,
                    self.deletes,
                );
                walk_side_queries(
                    self.sql,
                    &tables,
                    delete.selection.as_ref(),
                    ctes,
                    self.selects,
                    self.positions,
                );
                walk_returning(
                    self.sql,
                    delete.returning.as_deref(),
                    ctes,
                    self.positions,
                    self.selects,
                );
            }
            Statement::Query(query) => self.collect_query(query, ctes),
            Statement::Insert(insert) => {
                if let Some(source) = insert.source.as_deref() {
                    self.collect_query(source, ctes);
                }
                conflict::collect(
                    self.sql,
                    insert.on.as_ref(),
                    ctes,
                    self.selects,
                    self.positions,
                );
                walk_returning(
                    self.sql,
                    insert.returning.as_deref(),
                    ctes,
                    self.positions,
                    self.selects,
                );
            }
            Statement::Merge(statement) => {
                merge::collect(self.sql, statement, ctes, self.selects, self.positions)
            }
            _ => {}
        }
    }
}
