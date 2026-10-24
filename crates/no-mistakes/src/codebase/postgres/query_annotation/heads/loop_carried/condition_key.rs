//! Structural identities for pure tests that can correlate loop paths.
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

#[derive(Eq, Hash, PartialEq)]
pub(super) enum Key {
    Identifier(String),
    Boolean(bool),
    Number(u64),
    String(String),
    BigInt(String),
    Null,
    Unary(UnaryOperator, Box<Self>),
    Binary(BinaryOperator, Box<Self>, Box<Self>),
    Logical(u8, Box<Self>, Box<Self>),
}

pub(super) fn key(value: &Expression<'_>) -> Option<Key> {
    let mut value = unwrap_ts_wrappers(value);
    // A condition and its logical negation cannot take the same outcome.
    while let Expression::UnaryExpression(unary) = value {
        if unary.operator != UnaryOperator::LogicalNot {
            break;
        }
        value = unwrap_ts_wrappers(&unary.argument);
    }
    structural(value)
}

fn structural(value: &Expression<'_>) -> Option<Key> {
    match unwrap_ts_wrappers(value) {
        Expression::Identifier(value) => Some(Key::Identifier(value.name.to_string())),
        Expression::BooleanLiteral(value) => Some(Key::Boolean(value.value)),
        Expression::NumericLiteral(value) => Some(Key::Number(value.value.to_bits())),
        Expression::StringLiteral(value) => Some(Key::String(value.value.to_string())),
        Expression::BigIntLiteral(value) => Some(Key::BigInt(value.value.to_string())),
        Expression::NullLiteral(_) => Some(Key::Null),
        Expression::UnaryExpression(value) if value.operator != UnaryOperator::Delete => Some(
            Key::Unary(value.operator, Box::new(structural(&value.argument)?)),
        ),
        Expression::BinaryExpression(value)
            if !matches!(
                value.operator,
                BinaryOperator::In | BinaryOperator::Instanceof
            ) =>
        {
            Some(Key::Binary(
                value.operator,
                Box::new(structural(&value.left)?),
                Box::new(structural(&value.right)?),
            ))
        }
        Expression::LogicalExpression(value) => Some(Key::Logical(
            value.operator as u8,
            Box::new(structural(&value.left)?),
            Box::new(structural(&value.right)?),
        )),
        _ => None,
    }
}
