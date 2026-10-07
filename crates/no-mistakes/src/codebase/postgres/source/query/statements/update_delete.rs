use super::*;
use sqlparser::ast::{Delete, FromTable, Update, UpdateTableFromKind};
impl Collector<'_, '_> {
    pub(super) fn update(
        &mut self,
        value: &Update,
        scope: usize,
        env: &CteEnvironment,
    ) -> PostgresSqlQueryStatementKind {
        let target_relation_ids = self.from(&value.table, scope, env);
        let mut from_relation_ids = Vec::new();
        if let Some(
            UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables),
        ) = &value.from
        {
            for table in tables {
                from_relation_ids.extend(self.from(table, scope, env));
            }
        }
        let assignments = self.assignments(&value.assignments, scope, env);
        self.dml_predicate(value.selection.as_ref(), scope, env);
        if value.output.is_some()
            || value.or.is_some()
            || !value.order_by.is_empty()
            || value.limit.is_some()
            || !value.optimizer_hints.is_empty()
        {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "UPDATE modifiers",
                value.span(),
            );
        }
        PostgresSqlQueryStatementKind::Update {
            update: PostgresSqlCteUpdate {
                target_relation_ids,
                from_relation_ids,
                assignments,
                predicate: value
                    .selection
                    .as_ref()
                    .map(|expr| expression(expr, self.locations)),
            },
        }
    }
    pub(super) fn delete(
        &mut self,
        value: &Delete,
        scope: usize,
        env: &CteEnvironment,
    ) -> PostgresSqlQueryStatementKind {
        let mut target_relation_ids = Vec::new();
        let (FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables)) = &value.from;
        for table in tables {
            target_relation_ids.extend(self.from(table, scope, env));
        }
        let mut using_relation_ids = Vec::new();
        for table in value.using.iter().flatten() {
            using_relation_ids.extend(self.from(table, scope, env));
        }
        self.dml_predicate(value.selection.as_ref(), scope, env);
        if value.output.is_some()
            || !value.tables.is_empty()
            || !value.order_by.is_empty()
            || value.limit.is_some()
            || !value.optimizer_hints.is_empty()
        {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "DELETE modifiers",
                value.span(),
            );
        }
        PostgresSqlQueryStatementKind::Delete {
            delete: PostgresSqlCteDelete {
                target_relation_ids,
                using_relation_ids,
                predicate: value
                    .selection
                    .as_ref()
                    .map(|expr| expression(expr, self.locations)),
            },
        }
    }
}
