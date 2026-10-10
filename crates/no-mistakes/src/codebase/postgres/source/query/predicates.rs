use super::*;
use sqlparser::ast::{BinaryOperator, Expr, UnaryOperator};
mod functions;
impl Collector<'_, '_> {
    pub(super) fn expr(
        &mut self,
        expr: &Expr,
        scope: usize,
        clause: PostgresSqlQueryClause,
        join: Option<usize>,
        context: PostgresSqlPredicateContext,
        env: &CteEnvironment,
    ) {
        if let Some(column) = self.column(expr, scope, clause) {
            self.facts.columns.push(column);
            return;
        }
        match expr {
            Expr::Value(_) | Expr::TypedString(_) => {}
            Expr::Nested(inner) => self.expr(inner, scope, clause, join, context, env),
            Expr::BinaryOp { left, op, right } => {
                if *op == BinaryOperator::Eq {
                    self.facts.equalities.push(PostgresSqlQueryEquality {
                        scope_id: scope,
                        clause,
                        join_id: join,
                        left: self.column(left, scope, clause),
                        right: self.column(right, scope, clause),
                        context: context.clone(),
                        span: self.locations.span(expr.span()),
                    });
                }
                let mut child = context;
                match op {
                    BinaryOperator::And => {}
                    BinaryOperator::Or => {
                        child.mandatory = false;
                        child.under_or = true;
                    }
                    _ => {
                        child.mandatory = false;
                        child.under_other = true;
                    }
                }
                self.expr(left, scope, clause, join, child.clone(), env);
                self.expr(right, scope, clause, join, child, env);
            }
            Expr::UnaryOp { op, expr } => {
                let mut child = context;
                child.mandatory = false;
                let not = *op == UnaryOperator::Not;
                if not {
                    child.under_not = true;
                    self.not_depth += 1;
                } else {
                    child.under_other = true;
                }
                self.expr(expr, scope, clause, join, child, env);
                self.not_depth -= u32::from(not);
            }
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                let mut child = context;
                child.mandatory = false;
                child.under_case = true;
                for expr in operand
                    .iter()
                    .map(|x| x.as_ref())
                    .chain(conditions.iter().flat_map(|c| [&c.condition, &c.result]))
                    .chain(else_result.iter().map(|x| x.as_ref()))
                {
                    self.expr(expr, scope, clause, join, child.clone(), env);
                }
            }
            Expr::IsTrue(inner)
            | Expr::IsFalse(inner)
            | Expr::IsNotTrue(inner)
            | Expr::IsNotFalse(inner)
            | Expr::IsUnknown(inner)
            | Expr::IsNotUnknown(inner) => {
                let mut child = context;
                child.mandatory = false;
                child.under_boolean_test = true;
                self.expr(inner, scope, clause, join, child, env);
            }
            Expr::IsNull(inner) | Expr::IsNotNull(inner) | Expr::Cast { expr: inner, .. } => {
                let mut child = context;
                child.mandatory = false;
                child.under_other = true;
                self.expr(inner, scope, clause, join, child, env);
            }
            Expr::Exists { subquery, negated } => {
                // Wrapping NOT is already in `not_depth`. The node's own flag is separate.
                let not_depth = self.not_depth + u32::from(*negated);
                let start = self.facts.columns.len();
                let first_scope = self.facts.scopes.len();
                let child = self.query(
                    subquery,
                    Some(scope),
                    Some(scope),
                    clause,
                    env,
                    self.facts.scopes[scope].cte_definition_id,
                );
                let correlations: Vec<_> = self.facts.columns[start..]
                    .iter()
                    .filter(|c| c.relation_scope_id.is_some_and(|s| s < first_scope))
                    .cloned()
                    .collect();
                let mut context = context;
                if *negated {
                    context.mandatory = false;
                    context.under_not = true;
                }
                self.facts.exists.push(PostgresSqlQueryExists {
                    scope_id: scope,
                    subquery_scope_id: child,
                    negated: *negated,
                    not_depth,
                    effective_negated: not_depth % 2 == 1,
                    context,
                    correlated: !correlations.is_empty(),
                    correlations,
                    span: self.locations.span(expr.span()),
                });
            }
            Expr::Subquery(query) => {
                self.query(
                    query,
                    Some(scope),
                    Some(scope),
                    clause,
                    env,
                    self.facts.scopes[scope].cte_definition_id,
                );
            }
            Expr::InSubquery { expr, subquery, .. } => {
                self.nonpredicate(expr, scope, clause, env);
                self.query(
                    subquery,
                    Some(scope),
                    Some(scope),
                    clause,
                    env,
                    self.facts.scopes[scope].cte_definition_id,
                );
            }
            Expr::Function(function) => {
                self.function_expressions(function, scope, clause, join, context, env)
            }
            Expr::Tuple(exprs) | Expr::Array(sqlparser::ast::Array { elem: exprs, .. }) => {
                for expr in exprs {
                    self.nonpredicate(expr, scope, clause, env);
                }
            }
            Expr::InList { expr, list, .. } => {
                self.nonpredicate(expr, scope, clause, env);
                for expr in list {
                    self.nonpredicate(expr, scope, clause, env);
                }
            }
            Expr::Between {
                expr, low, high, ..
            } => {
                for expr in [expr, low, high] {
                    self.nonpredicate(expr, scope, clause, env);
                }
            }
            _ => self.unsupported(scope, clause, "expression form", expr.span()),
        }
    }
}
