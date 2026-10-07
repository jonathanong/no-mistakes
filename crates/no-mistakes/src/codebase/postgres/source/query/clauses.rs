use super::*;
use sqlparser::ast::{LimitClause, OrderByKind};
impl Collector<'_, '_> {
    pub(super) fn query_clauses(&mut self, query: &Query, scope: usize, env: &CteEnvironment) {
        // Keep ordinary SELECT projection stable while diagnosing omitted source locks.
        if self.insert_source && !query.locks.is_empty() {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "INSERT source locking",
                query.span(),
            );
        }
        if let Some(order) = &query.order_by {
            match &order.kind {
                OrderByKind::Expressions(exprs) => {
                    for expr in exprs {
                        self.nonpredicate(&expr.expr, scope, PostgresSqlQueryClause::OrderBy, env);
                    }
                }
                _ => self.unsupported(
                    scope,
                    PostgresSqlQueryClause::OrderBy,
                    "ordering form",
                    query.span(),
                ),
            }
        }
        if let Some(limit) = &query.limit_clause {
            match limit {
                LimitClause::LimitOffset {
                    limit,
                    offset,
                    limit_by,
                } => {
                    if let Some(expr) = limit {
                        self.nonpredicate(expr, scope, PostgresSqlQueryClause::Limit, env);
                    }
                    if let Some(offset) = offset {
                        self.nonpredicate(
                            &offset.value,
                            scope,
                            PostgresSqlQueryClause::Offset,
                            env,
                        );
                    }
                    for expr in limit_by {
                        self.nonpredicate(expr, scope, PostgresSqlQueryClause::Limit, env);
                    }
                }
                LimitClause::OffsetCommaLimit { offset, limit } => {
                    self.nonpredicate(offset, scope, PostgresSqlQueryClause::Offset, env);
                    self.nonpredicate(limit, scope, PostgresSqlQueryClause::Limit, env);
                }
            }
        }
        if let Some(fetch) = &query.fetch {
            if let Some(expr) = &fetch.quantity {
                self.nonpredicate(expr, scope, PostgresSqlQueryClause::Limit, env);
            }
        }
        if [
            query.for_clause.is_some(),
            query.settings.is_some(),
            !query.pipe_operators.is_empty(),
        ]
        .into_iter()
        .any(|present| present)
        {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "query extension",
                query.span(),
            );
        }
    }
}
