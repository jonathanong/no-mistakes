use crate::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, CaseWhen, DataType, Expr, UnaryOperator, Value};
mod calls;
mod literals;
use calls::function_result;
use literals::{negated_literal_type, number_type};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BuiltinType {
    SmallInt,
    Integer,
    BigInt,
    Numeric,
    Real,
    DoublePrecision,
    Text,
    Bool,
    Date,
}

impl BuiltinType {
    fn numeric(self) -> bool {
        self.integer() || self.float() || self == Self::Numeric
    }

    fn integer(self) -> bool {
        matches!(self, Self::SmallInt | Self::Integer | Self::BigInt)
    }

    fn float(self) -> bool {
        matches!(self, Self::Real | Self::DoublePrecision)
    }

    fn comparable_to(self, other: Self) -> bool {
        self == other || self.integer() && other.integer() || self.float() && other.float()
    }
}

pub(super) fn binary_is_proven(op: &BinaryOperator, left: &Expr, right: &Expr) -> bool {
    binary_result(op, left, right).is_some()
}

pub(super) fn unary_is_proven(op: UnaryOperator, expr: &Expr) -> bool {
    unary_result(op, expr).is_some()
}

pub(super) fn comparable(left: &Expr, right: &Expr) -> bool {
    operand_type(left)
        .is_some_and(|left| operand_type(right).is_some_and(|right| left.comparable_to(right)))
}

pub(super) fn list_operands_are_proven(expr: &Expr, items: &[Expr]) -> bool {
    operand_type(expr).is_some_and(|kind| items.iter().all(|item| operand_type(item) == Some(kind)))
}

pub(super) fn pattern_is_proven(expr: &Expr, pattern: &Expr, escape: Option<&Expr>) -> bool {
    [Some(expr), Some(pattern), escape]
        .into_iter()
        .flatten()
        .all(is_proven_text)
}

pub(super) fn is_proven_text(expr: &Expr) -> bool {
    operand_type(expr) == Some(BuiltinType::Text)
}

pub(super) fn is_proven_boolean(expr: &Expr) -> bool {
    operand_type(expr) == Some(BuiltinType::Bool)
}

pub(super) fn array_elements_are_proven(elements: &[Expr]) -> bool {
    if let Some((first, rest)) = elements.split_first() {
        list_operands_are_proven(first, rest)
    } else {
        false
    }
}

pub(super) fn substring_is_proven(
    expr: &Expr,
    from: Option<&Expr>,
    length: Option<&Expr>,
    allows_pattern: bool,
) -> bool {
    if operand_type(expr) != Some(BuiltinType::Text) {
        return false;
    }
    let length_type = length.and_then(operand_type);
    match from.and_then(operand_type) {
        Some(kind)
            if kind == BuiltinType::Integer || allows_pattern && kind == BuiltinType::Text =>
        {
            length.is_none() || length_type == Some(kind)
        }
        _ => from.is_none() && length_type == Some(BuiltinType::Integer),
    }
}

pub(super) fn case_is_proven(
    operand: Option<&Expr>,
    arms: &[CaseWhen],
    fallback: Option<&Expr>,
) -> bool {
    let condition_type = operand.map_or(Some(BuiltinType::Bool), operand_type);
    let result_type = arms.first().and_then(|arm| operand_type(&arm.result));
    condition_type.is_some()
        && result_type.is_some()
        && arms.iter().all(|arm| {
            operand_type(&arm.condition) == condition_type
                && operand_type(&arm.result) == result_type
        })
        && fallback.is_none_or(|expr| operand_type(expr) == result_type)
}

fn operand_type(expr: &Expr) -> Option<BuiltinType> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::Number(number, _) => Some(number_type(number, false)),
            Value::Boolean(_) => Some(BuiltinType::Bool),
            Value::SingleQuotedString(_) | Value::EscapedStringLiteral(_) => {
                Some(BuiltinType::Text)
            }
            _ => None,
        },
        Expr::Cast { data_type, .. } => type_name(data_type),
        Expr::TypedString(typed) => type_name(&typed.data_type),
        Expr::IsNull(_) | Expr::IsNotNull(_) => Some(BuiltinType::Bool),
        Expr::Function(function) => function_result(function),
        Expr::BinaryOp { left, op, right } => binary_result(op, left, right),
        Expr::UnaryOp { op, expr } => unary_result(*op, expr),
        _ => None,
    }
}

fn type_name(data_type: &DataType) -> Option<BuiltinType> {
    match data_type {
        DataType::SmallInt(_) | DataType::Int2(_) => Some(BuiltinType::SmallInt),
        DataType::Integer(_) | DataType::Int(_) | DataType::Int4(_) => Some(BuiltinType::Integer),
        DataType::BigInt(_) | DataType::Int8(_) => Some(BuiltinType::BigInt),
        DataType::Numeric(_) | DataType::Decimal(_) => Some(BuiltinType::Numeric),
        DataType::Real => Some(BuiltinType::Real),
        DataType::DoublePrecision => Some(BuiltinType::DoublePrecision),
        DataType::Text => Some(BuiltinType::Text),
        DataType::Boolean | DataType::Bool => Some(BuiltinType::Bool),
        DataType::Date => Some(BuiltinType::Date),
        _ => None,
    }
}

fn binary_result(op: &BinaryOperator, left: &Expr, right: &Expr) -> Option<BuiltinType> {
    use BinaryOperator::*;
    let left = operand_type(left)?;
    let right = operand_type(right)?;
    if !left.comparable_to(right) {
        return None;
    }
    match op {
        Plus | Minus | Multiply | Divide if left.numeric() => Some(left.max(right)),
        Modulo if left == right && (left.integer() || left == BuiltinType::Numeric) => Some(left),
        Eq | NotEq | Gt | GtEq | Lt | LtEq => Some(BuiltinType::Bool),
        And | Or if left == BuiltinType::Bool => Some(BuiltinType::Bool),
        StringConcat if left == BuiltinType::Text => Some(BuiltinType::Text),
        _ => None,
    }
}

fn unary_result(op: UnaryOperator, expr: &Expr) -> Option<BuiltinType> {
    if op == UnaryOperator::Minus {
        if let Some(kind) = negated_literal_type(expr) {
            return Some(kind);
        }
    }
    let kind = operand_type(expr)?;
    match op {
        UnaryOperator::Plus | UnaryOperator::Minus if kind.numeric() => Some(kind),
        UnaryOperator::Not if kind == BuiltinType::Bool => Some(kind),
        UnaryOperator::BitwiseNot if kind.integer() => Some(kind),
        _ => None,
    }
}
