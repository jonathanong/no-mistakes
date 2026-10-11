use super::*;
use sqlparser::ast::{Distinct, GroupByExpr, Select, SelectItem, SetExpr};
impl Collector<'_, '_> {
    pub(super) fn body(
        &mut self,
        body: &SetExpr,
        scope: usize,
        env: &CteEnvironment,
        body_end: Option<usize>,
    ) {
        match body {
            SetExpr::Select(select) => self.select(select, scope, env),
            SetExpr::Query(query) => {
                self.query(
                    query,
                    Some(scope),
                    self.states[scope].visible_parent,
                    PostgresSqlQueryClause::SetBranch,
                    env,
                    self.facts.scopes[scope].cte_definition_id,
                );
            }
            SetExpr::SetOperation {
                left,
                right,
                op,
                set_quantifier,
            } => {
                self.facts.scopes[scope].set_operation = Some(op.to_string());
                self.facts.scopes[scope].set_quantifier = Some(set_quantifier.to_string());
                let (left_span, right_span) = self.span_bounds.set_branches(
                    left,
                    right,
                    self.facts.scopes[scope].span.as_ref(),
                    body_end,
                    self.locations,
                );
                for (branch, span) in [(left, left_span), (right, right_span)] {
                    let child = self.scope(
                        Some(scope),
                        self.states[scope].visible_parent,
                        PostgresSqlQueryClause::SetBranch,
                        self.facts.scopes[scope].cte_definition_id,
                        span.or_else(|| self.locations.span(branch.span())),
                    );
                    let child_end = self.facts.scopes[child]
                        .span
                        .as_ref()
                        .map(|span| span.end.offset);
                    self.body(branch, child, env, child_end);
                }
            }
            SetExpr::Values(values) => {
                for row in &values.rows {
                    for expr in row.iter() {
                        self.nonpredicate(expr, scope, PostgresSqlQueryClause::Other, env);
                    }
                }
            }
            SetExpr::Insert(statement)
            | SetExpr::Update(statement)
            | SetExpr::Delete(statement)
            | SetExpr::Merge(statement) => {
                self.statement(statement, scope, env);
            }
            _ => self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "query body",
                body.span(),
            ),
        }
    }
    pub(super) fn select(&mut self, select: &Select, scope: usize, env: &CteEnvironment) {
        for table in &select.from {
            self.from(table, scope, env);
        }
        for item in &select.projection {
            match item {
                SelectItem::UnnamedExpr(expr)
                | SelectItem::ExprWithAlias { expr, .. }
                | SelectItem::ExprWithAliases { expr, .. } => {
                    self.nonpredicate(expr, scope, PostgresSqlQueryClause::Projection, env)
                }
                SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(_, _) => {}
            }
        }
        if let Some(Distinct::On(expressions)) = &select.distinct {
            for expr in expressions {
                self.nonpredicate(expr, scope, PostgresSqlQueryClause::Projection, env);
            }
        }
        if let Some(expr) = &select.selection {
            self.expr(
                expr,
                scope,
                PostgresSqlQueryClause::Where,
                None,
                Self::predicate_context(true),
                env,
            );
        }
        if let Some(expr) = &select.having {
            self.nonpredicate(expr, scope, PostgresSqlQueryClause::Having, env);
        }
        match &select.group_by {
            GroupByExpr::Expressions(exprs, _) => {
                for expr in exprs {
                    self.nonpredicate(expr, scope, PostgresSqlQueryClause::GroupBy, env);
                }
            }
            _ => self.unsupported(
                scope,
                PostgresSqlQueryClause::GroupBy,
                "grouping form",
                select.span(),
            ),
        }
        if [
            !select.named_window.is_empty(),
            select.qualify.is_some(),
            select.prewhere.is_some(),
            !select.lateral_views.is_empty(),
            !select.connect_by.is_empty(),
            select.top.is_some(),
            !select.cluster_by.is_empty(),
            !select.distribute_by.is_empty(),
            !select.sort_by.is_empty(),
        ]
        .into_iter()
        .any(|present| present)
        {
            self.unsupported(
                scope,
                PostgresSqlQueryClause::Other,
                "select extension",
                select.span(),
            );
        }
    }
    pub(super) fn nonpredicate(
        &mut self,
        expr: &sqlparser::ast::Expr,
        scope: usize,
        clause: PostgresSqlQueryClause,
        env: &CteEnvironment,
    ) {
        self.expr(
            expr,
            scope,
            clause,
            None,
            Self::predicate_context(false),
            env,
        );
    }
}
