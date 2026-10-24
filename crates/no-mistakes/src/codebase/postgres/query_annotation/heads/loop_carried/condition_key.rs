//! Structural identities for pure tests that can correlate loop paths.
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

pub(in crate::codebase::postgres::query_annotation::heads) struct Condition {
    pub(in crate::codebase::postgres::query_annotation::heads::loop_carried) key: Key,
    pub(in crate::codebase::postgres::query_annotation::heads::loop_carried) negative: bool,
    pub(in crate::codebase::postgres::query_annotation::heads) references: Vec<Span>,
}

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

pub(super) fn key(value: &Expression<'_>) -> Option<Condition> {
    let mut value = unwrap_ts_wrappers(value);
    let mut negative = false;
    while let Expression::UnaryExpression(unary) = value {
        if unary.operator != UnaryOperator::LogicalNot {
            break;
        }
        negative = !negative;
        value = unwrap_ts_wrappers(&unary.argument);
    }
    let mut references = Vec::new();
    let key = structural(value, &mut references)?;
    Some(Condition {
        key,
        negative,
        references,
    })
}

fn structural(value: &Expression<'_>, references: &mut Vec<Span>) -> Option<Key> {
    match unwrap_ts_wrappers(value) {
        Expression::Identifier(value) => {
            references.push((value.span.start, value.span.end));
            Some(Key::Identifier(value.name.to_string()))
        }
        Expression::BooleanLiteral(value) => Some(Key::Boolean(value.value)),
        // This is structural identity, not JavaScript numeric equality:
        // +0/-0 differ, while matching NaN bit patterns get the same key.
        Expression::NumericLiteral(value) => Some(Key::Number(value.value.to_bits())),
        Expression::StringLiteral(value) => Some(Key::String(value.value.to_string())),
        Expression::BigIntLiteral(value) => Some(Key::BigInt(value.value.to_string())),
        Expression::NullLiteral(_) => Some(Key::Null),
        Expression::UnaryExpression(value) if value.operator != UnaryOperator::Delete => {
            Some(Key::Unary(
                value.operator,
                Box::new(structural(&value.argument, references)?),
            ))
        }
        Expression::BinaryExpression(value)
            if !matches!(
                value.operator,
                BinaryOperator::In | BinaryOperator::Instanceof
            ) =>
        {
            Some(Key::Binary(
                value.operator,
                Box::new(structural(&value.left, references)?),
                Box::new(structural(&value.right, references)?),
            ))
        }
        Expression::LogicalExpression(value) => Some(Key::Logical(
            value.operator as u8,
            Box::new(structural(&value.left, references)?),
            Box::new(structural(&value.right, references)?),
        )),
        _ => None,
    }
}
