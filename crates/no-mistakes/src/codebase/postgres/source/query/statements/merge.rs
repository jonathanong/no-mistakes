use super::*;
use sqlparser::ast::{Merge, MergeAction, MergeInsertKind, MergeUpdateKind};
impl Collector<'_, '_> {
    pub(super) fn merge(
        &mut self,
        value: &Merge,
        scope: usize,
        env: &CteEnvironment,
    ) -> PostgresSqlQueryStatementKind {
        let target_relation_ids = self.relation(&value.table, scope, env);
        let source_relation_ids = self.relation(&value.source, scope, env);
        // Match conditions choose actions; they are not mandatory row filters.
        self.nonpredicate(&value.on, scope, PostgresSqlQueryClause::Other, env);
        if matches!(
            value.output,
            Some(sqlparser::ast::OutputClause::Output { .. })
        ) || !value.optimizer_hints.is_empty()
        {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "MERGE modifiers",
                value.span(),
            );
        }
        let clauses = value
            .clauses
            .iter()
            .map(|clause| {
                if let Some(predicate) = &clause.predicate {
                    self.nonpredicate(predicate, scope, PostgresSqlQueryClause::Other, env);
                }
                let action = match &clause.action {
                    MergeAction::Insert(insert) => match &insert.kind {
                        MergeInsertKind::Values(values) if insert.insert_predicate.is_none() => {
                            let rows = values
                                .rows
                                .iter()
                                .map(|row| {
                                    row.iter()
                                        .map(|expr| {
                                            self.nonpredicate(
                                                expr,
                                                scope,
                                                PostgresSqlQueryClause::Other,
                                                env,
                                            );
                                            expression(expr, self.locations)
                                        })
                                        .collect()
                                })
                                .collect();
                            PostgresSqlMergeAction::Insert {
                                columns: insert.columns.iter().map(name).collect(),
                                rows,
                            }
                        }
                        _ => {
                            self.unsupported(
                                scope,
                                PostgresSqlQueryClause::Other,
                                "MERGE INSERT action",
                                clause.span(),
                            );
                            PostgresSqlMergeAction::Unsupported {
                                reason: "MERGE INSERT action".into(),
                            }
                        }
                    },
                    MergeAction::Update(update) => match &update.kind {
                        MergeUpdateKind::Set(assignments)
                            if update.delete_predicate.is_none()
                                && update.update_predicate.is_none() =>
                        {
                            PostgresSqlMergeAction::Update {
                                assignments: self.assignments(assignments, scope, env),
                            }
                        }
                        _ => {
                            self.unsupported(
                                scope,
                                PostgresSqlQueryClause::Other,
                                "MERGE UPDATE action",
                                clause.span(),
                            );
                            PostgresSqlMergeAction::Unsupported {
                                reason: "MERGE UPDATE action".into(),
                            }
                        }
                    },
                    MergeAction::Delete { .. } => PostgresSqlMergeAction::Delete,
                    MergeAction::DoNothing { .. } => PostgresSqlMergeAction::DoNothing,
                };
                PostgresSqlMergeClause {
                    match_kind: clause.clause_kind.to_string(),
                    predicate: clause
                        .predicate
                        .as_ref()
                        .map(|expr| expression(expr, self.locations)),
                    span: self.locations.span(clause.span()),
                    action,
                }
            })
            .collect();
        PostgresSqlQueryStatementKind::Merge {
            merge: PostgresSqlCteMerge {
                target_relation_ids,
                source_relation_ids,
                predicate: expression(&value.on, self.locations),
                clauses,
            },
        }
    }
}
