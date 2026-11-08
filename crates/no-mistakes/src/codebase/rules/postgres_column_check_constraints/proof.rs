use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{BinaryOperator, Expr, TableConstraint, UnaryOperator, Value};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;
use std::collections::BTreeSet;

pub(super) fn proven_columns(definition: &str) -> BTreeSet<String> {
    let Some(expr) = check_expression(definition) else {
        return BTreeSet::new();
    };
    let mut columns = BTreeSet::new();
    collect(&expr, &mut columns);
    columns
}

fn check_expression(definition: &str) -> Option<Expr> {
    let mut parser = Parser::new(&PostgreSqlDialect {})
        .try_with_sql(definition)
        .ok()?;
    let TableConstraint::Check(check) = parser.parse_optional_table_constraint().ok()?? else {
        return None;
    };
    (parser.peek_token().token == Token::EOF && check.enforced != Some(false))
        .then_some(*check.expr)
}

fn collect(expr: &Expr, columns: &mut BTreeSet<String>) {
    let Expr::BinaryOp { left, op, right } = peel(expr) else {
        return;
    };
    if *op == BinaryOperator::And {
        collect(left, columns);
        collect(right, columns);
        return;
    }
    let candidate = match op {
        BinaryOperator::GtEq if number(right).is_some_and(|bound| bound >= 0) => column(left),
        BinaryOperator::LtEq if number(left).is_some_and(|bound| bound >= 0) => column(right),
        BinaryOperator::Gt if number(right).is_some_and(|bound| bound >= -1) => column(left),
        BinaryOperator::Lt if number(left).is_some_and(|bound| bound >= -1) => column(right),
        BinaryOperator::Eq if number(right).is_some_and(|bound| bound >= 0) => column(left),
        BinaryOperator::Eq if number(left).is_some_and(|bound| bound >= 0) => column(right),
        _ => None,
    };
    columns.extend(candidate);
}

fn column(expr: &Expr) -> Option<String> {
    match peel(expr) {
        Expr::Identifier(ident) => Some(ident_key(ident)),
        _ => None,
    }
}

fn number(expr: &Expr) -> Option<i64> {
    match peel(expr) {
        Expr::Value(value) => match &value.value {
            Value::Number(value, false) => value.parse().ok(),
            _ => None,
        },
        Expr::UnaryOp {
            op: UnaryOperator::Minus,
            expr,
        } => number(expr)?.checked_neg(),
        Expr::Cast {
            expr,
            data_type,
            format: None,
            ..
        } if super::compile::integer_type(&data_type.to_string()).is_some() => match peel(expr) {
            Expr::Value(value) => match &value.value {
                Value::SingleQuotedString(value) => value.parse().ok(),
                _ => number(expr),
            },
            _ => number(expr),
        },
        _ => None,
    }
}

fn peel(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => peel(inner),
        other => other,
    }
}
