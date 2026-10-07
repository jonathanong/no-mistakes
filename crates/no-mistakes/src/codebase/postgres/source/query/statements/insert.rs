use super::*;
use sqlparser::ast::{Insert, OnConflictAction, OnInsert, SetExpr};
impl Collector<'_, '_> {
    pub(super) fn insert(
        &mut self,
        value: &Insert,
        scope: usize,
        env: &CteEnvironment,
    ) -> PostgresSqlQueryStatementKind {
        // Reuse conflict/modifier projection without creating another query collector.
        // RETURNING and source queries are projected below in this report's scopes.
        let core =
            crate::codebase::postgres::source::insert::project_cte_core(value, self.locations);
        if !core.complete {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "INSERT modifiers or conflict",
                value.span(),
            );
        }
        let source = match value.source.as_deref() {
            None => PostgresSqlCteInsertSource::DefaultValues,
            Some(query) if matches!(&*query.body, SetExpr::Values(_)) && query.with.is_none() => {
                let SetExpr::Values(values) = &*query.body else {
                    unreachable!()
                };
                if !crate::codebase::postgres::source::insert::values_unmodified(query) {
                    self.unsupported(
                        scope,
                        PostgresSqlQueryClause::Other,
                        "INSERT VALUES query modifiers",
                        query.span(),
                    );
                }
                self.query_clauses(query, scope, env);
                let rows = values
                    .rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|expr| {
                                self.nonpredicate(expr, scope, PostgresSqlQueryClause::Other, env);
                                expression(expr, self.locations)
                            })
                            .collect()
                    })
                    .collect();
                PostgresSqlCteInsertSource::Values {
                    rows,
                    span: self.locations.span(query.span()),
                }
            }
            Some(query) => {
                let outer_insert_source = self.insert_source;
                self.insert_source = true;
                let query_scope_id = self.query(
                    query,
                    Some(scope),
                    None,
                    PostgresSqlQueryClause::Other,
                    env,
                    self.facts.scopes[scope].cte_definition_id,
                );
                self.insert_source = outer_insert_source;
                PostgresSqlCteInsertSource::Select {
                    query_scope_id,
                    span: self.locations.span(query.span()),
                }
            }
        };
        // Target aliases are visible to conflict/RETURNING, not to the INSERT source.
        if let Some(table) = &core.table {
            self.register(PostgresSqlQueryRelation {
                id: self.facts.relations.len(),
                scope_id: scope,
                kind: PostgresSqlQueryRelationKind::Table,
                name: Some(table.clone()),
                alias: core.alias.clone(),
                column_aliases: Vec::new(),
                cte_id: None,
                subquery_scope_id: None,
                members: Vec::new(),
                lateral: false,
                span: self.locations.span(value.table.span()),
            });
        }
        // Conflict actions are conditional; their predicates are not global row filters.
        if let Some(OnInsert::OnConflict(conflict)) = &value.on {
            if let OnConflictAction::DoUpdate(update) = &conflict.action {
                for assignment in &update.assignments {
                    self.nonpredicate(&assignment.value, scope, PostgresSqlQueryClause::Other, env);
                }
                if let Some(predicate) = &update.selection {
                    self.nonpredicate(predicate, scope, PostgresSqlQueryClause::Other, env);
                }
            }
        }
        PostgresSqlQueryStatementKind::Insert {
            insert: Box::new(PostgresSqlCteInsert {
                table: core.table,
                alias: core.alias,
                columns: core.columns,
                columns_omitted: core.columns_omitted,
                source,
                on_conflict: core.on_conflict,
                diagnostics: core.diagnostics,
            }),
        }
    }
}
