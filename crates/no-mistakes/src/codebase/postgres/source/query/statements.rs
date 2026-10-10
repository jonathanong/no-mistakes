use super::*;
use crate::codebase::postgres::source::expressions::expression;
use sqlparser::ast::{
    Assignment, AssignmentTarget, SelectItem, SelectItemQualifiedWildcardKind, Statement,
};
mod insert;
mod merge;
mod update_delete;

impl Collector<'_, '_> {
    pub(super) fn statement(&mut self, statement: &Statement, scope: usize, env: &CteEnvironment) {
        let before = self.facts.unsupported.len();
        let (facts, returning) = match statement {
            Statement::Insert(value) => {
                (self.insert(value, scope, env), value.returning.as_deref())
            }
            Statement::Update(value) => {
                (self.update(value, scope, env), value.returning.as_deref())
            }
            Statement::Delete(value) => {
                (self.delete(value, scope, env), value.returning.as_deref())
            }
            Statement::Merge(value) => {
                let returning = match &value.output {
                    Some(sqlparser::ast::OutputClause::Returning { select_items, .. }) => {
                        Some(select_items.as_slice())
                    }
                    _ => None,
                };
                (self.merge(value, scope, env), returning)
            }
            _ => {
                self.unsupported(
                    scope,
                    PostgresSqlQueryClause::Other,
                    "data-modifying query body",
                    statement.span(),
                );
                (
                    PostgresSqlQueryStatementKind::Unsupported {
                        reason: "data-modifying query body".into(),
                    },
                    None,
                )
            }
        };
        let returning = self.returning(returning.unwrap_or_default(), scope, env);
        let span = self.locations.span(statement.span());
        if span.is_none() {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "data-modifying statement source span",
                statement.span(),
            );
        }
        let unsupported = self.facts.unsupported[before..].to_vec();
        let sql = span
            .as_ref()
            .map(|span| self.locations.slice(span).to_owned())
            .unwrap_or_default();
        let query_scope = &self.facts.scopes[scope];
        self.facts
            .nested_statements
            .push(PostgresSqlQueryStatement {
                ordinal: 0,
                cte_id: query_scope.cte_definition_id,
                query_scope_id: scope,
                parent_scope_id: query_scope.parent_scope_id,
                sql,
                span,
                complete: unsupported.is_empty(),
                unsupported,
                returning,
                facts,
            });
    }

    fn dml_predicate(
        &mut self,
        predicate: Option<&sqlparser::ast::Expr>,
        scope: usize,
        env: &CteEnvironment,
    ) {
        if let Some(expr) = predicate {
            self.expr(
                expr,
                scope,
                PostgresSqlQueryClause::Where,
                None,
                Self::predicate_context(true),
                env,
            );
        }
    }
    fn assignments(
        &mut self,
        assignments: &[Assignment],
        scope: usize,
        env: &CteEnvironment,
    ) -> Vec<PostgresSqlDmlAssignment> {
        assignments
            .iter()
            .map(|value| {
                self.nonpredicate(&value.value, scope, PostgresSqlQueryClause::Other, env);
                let columns = match &value.target {
                    AssignmentTarget::ColumnName(column) => vec![name(column)],
                    AssignmentTarget::Tuple(columns) => columns.iter().map(name).collect(),
                };
                PostgresSqlDmlAssignment {
                    columns,
                    expression: expression(&value.value, self.locations),
                    span: self.locations.span(value.span()),
                }
            })
            .collect()
    }
    fn returning(
        &mut self,
        items: &[SelectItem],
        scope: usize,
        env: &CteEnvironment,
    ) -> Vec<PostgresSqlReturningItem> {
        items
            .iter()
            .map(|item| match item {
                SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                    self.nonpredicate(expr, scope, PostgresSqlQueryClause::Projection, env);
                    let alias = if let SelectItem::ExprWithAlias { alias, .. } = item {
                        Some(identifier(alias))
                    } else {
                        None
                    };
                    let projected = expression(expr, self.locations);
                    // CTE-core skips `returning::project`; incompleteness is recorded here.
                    if !projected.children_complete {
                        self.unsupported(
                            scope,
                            PostgresSqlQueryClause::Projection,
                            "unsupported or incompletely represented syntax",
                            expr.span(),
                        );
                    }
                    PostgresSqlReturningItem::Expression {
                        expression: Box::new(projected),
                        alias,
                    }
                }
                SelectItem::Wildcard(options)
                | SelectItem::QualifiedWildcard(
                    SelectItemQualifiedWildcardKind::ObjectName(_),
                    options,
                ) if *options == Default::default() => {
                    let qualifier = if let SelectItem::QualifiedWildcard(
                        SelectItemQualifiedWildcardKind::ObjectName(name),
                        _,
                    ) = item
                    {
                        Some(super::name(name))
                    } else {
                        None
                    };
                    PostgresSqlReturningItem::Wildcard {
                        qualifier,
                        span: self.locations.span(item.span()),
                    }
                }
                _ => {
                    self.unsupported(
                        scope,
                        PostgresSqlQueryClause::Projection,
                        "RETURNING item",
                        item.span(),
                    );
                    PostgresSqlReturningItem::Unsupported {
                        reason: "RETURNING item".into(),
                        span: self.locations.span(item.span()),
                    }
                }
            })
            .collect()
    }
}
