use super::super::super::insert::spans::source_quantified_expression;
use super::*;

impl Collector<'_, '_> {
    pub(super) fn quantified(
        &mut self,
        expr: &Expr,
        (left, operator, right): (&Expr, &BinaryOperator, &Expr),
        (scope, clause, join): (usize, PostgresSqlQueryClause, Option<usize>),
        context: PostgresSqlPredicateContext,
        env: &CteEnvironment,
    ) {
        let left_facts =
            source_quantified_expression(left, &self.expression_delimiters, self.locations);
        let right_facts =
            source_quantified_expression(right, &self.expression_delimiters, self.locations);
        let complete = matches!(
            operator,
            BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq
        ) && operand(left, false, 0)
            && operand(right, true, 0)
            && left_facts.children_complete
            && right_facts.children_complete;
        self.facts
            .quantified_comparisons
            .push(PostgresSqlQueryQuantifiedComparison {
                scope_id: scope,
                clause,
                join_id: join,
                operator: operator.to_string(),
                quantifier: if matches!(expr, Expr::AllOp { .. }) {
                    PostgresSqlQuantifier::All
                } else {
                    PostgresSqlQuantifier::Any
                },
                left: left_facts,
                right: right_facts,
                context: context.clone(),
                span: source_quantified_expression(
                    expr,
                    &self.expression_delimiters,
                    self.locations,
                )
                .span,
                complete,
            });
        if !complete {
            self.unsupported(scope, clause, "scalar quantified comparison", expr.span());
        }
        // Operands are opaque to required-conjunct and equality inference.
        let child = opaque_context(context);
        self.expr(left, scope, clause, join, child.clone(), env);
        self.expr(right, scope, clause, join, child, env);
    }
}

fn operand(expr: &Expr, array: bool, depth: usize) -> bool {
    if depth >= 64 {
        return false;
    }
    match expr {
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) => true,
        Expr::Value(value) => {
            !array
                || matches!(
                    value.value,
                    sqlparser::ast::Value::Placeholder(_)
                        | sqlparser::ast::Value::Null
                        | sqlparser::ast::Value::SingleQuotedString(_)
                )
        }
        Expr::TypedString(value) => {
            !array || matches!(value.data_type, sqlparser::ast::DataType::Array(_))
        }
        Expr::Nested(inner) => operand(inner, array, depth + 1),
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } if !array => operand(expr, false, depth + 1),
        Expr::Cast {
            kind,
            expr,
            format,
            data_type,
            ..
        } => {
            matches!(
                kind,
                sqlparser::ast::CastKind::Cast | sqlparser::ast::CastKind::DoubleColon
            ) && matches!(data_type, sqlparser::ast::DataType::Array(_)) == array
                && format.is_none()
                && operand(expr, array, depth + 1)
        }
        Expr::Array(value) if array => value
            .elem
            .iter()
            .all(|expr| operand(expr, false, depth + 1)),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
