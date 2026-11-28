use super::function::{arguments_are_proven, pure_name};
use crate::idents::{function_arg_expr, unwrap_expr};
use sqlparser::ast::{
    BinaryOperator, DataType, Expr, Function, FunctionArguments, UnaryOperator, Value,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BuiltinType {
    Integer,
    Numeric,
    Float,
    Text,
    Bool,
    Date,
}

impl BuiltinType {
    fn numeric(self) -> bool {
        matches!(self, Self::Integer | Self::Numeric | Self::Float)
    }
}

pub(super) fn binary_is_proven(op: &BinaryOperator, left: &Expr, right: &Expr) -> bool {
    binary_result(op, left, right).is_some()
}

pub(super) fn unary_is_proven(op: UnaryOperator, expr: &Expr) -> bool {
    unary_result(op, expr).is_some()
}

pub(super) fn comparable(left: &Expr, right: &Expr) -> bool {
    operand_type(left).is_some_and(|left| operand_type(right) == Some(left))
}

pub(super) fn list_operands_are_proven(expr: &Expr, items: &[Expr]) -> bool {
    operand_type(expr).is_some_and(|kind| items.iter().all(|item| operand_type(item) == Some(kind)))
}

pub(super) fn pattern_is_proven(expr: &Expr, pattern: &Expr, escape: Option<&Expr>) -> bool {
    [Some(expr), Some(pattern), escape]
        .into_iter()
        .flatten()
        .all(|expr| operand_type(expr) == Some(BuiltinType::Text))
}

fn operand_type(expr: &Expr) -> Option<BuiltinType> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::Number(number, _) if number.contains(['.', 'e', 'E']) => {
                Some(BuiltinType::Numeric)
            }
            Value::Number(..) => Some(BuiltinType::Integer),
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
        DataType::SmallInt(_)
        | DataType::Integer(_)
        | DataType::Int(_)
        | DataType::BigInt(_)
        | DataType::Int2(_)
        | DataType::Int4(_)
        | DataType::Int8(_) => Some(BuiltinType::Integer),
        DataType::Numeric(_) | DataType::Decimal(_) => Some(BuiltinType::Numeric),
        DataType::Real | DataType::DoublePrecision => Some(BuiltinType::Float),
        DataType::Text | DataType::Varchar(_) => Some(BuiltinType::Text),
        DataType::Boolean | DataType::Bool => Some(BuiltinType::Bool),
        DataType::Date => Some(BuiltinType::Date),
        _ => None,
    }
}

fn binary_result(op: &BinaryOperator, left: &Expr, right: &Expr) -> Option<BuiltinType> {
    use BinaryOperator::*;
    let left = operand_type(left)?;
    let right = operand_type(right)?;
    if left != right {
        return None;
    }
    match op {
        Plus | Minus | Multiply | Divide if left.numeric() => Some(left),
        Modulo if matches!(left, BuiltinType::Integer | BuiltinType::Numeric) => Some(left),
        Eq | NotEq | Gt | GtEq | Lt | LtEq => Some(BuiltinType::Bool),
        And | Or if left == BuiltinType::Bool => Some(BuiltinType::Bool),
        StringConcat if left == BuiltinType::Text => Some(BuiltinType::Text),
        _ => None,
    }
}

fn unary_result(op: UnaryOperator, expr: &Expr) -> Option<BuiltinType> {
    let kind = operand_type(expr)?;
    match op {
        UnaryOperator::Plus | UnaryOperator::Minus if kind.numeric() => Some(kind),
        UnaryOperator::Not if kind == BuiltinType::Bool => Some(kind),
        UnaryOperator::BitwiseNot if kind == BuiltinType::Integer => Some(kind),
        _ => None,
    }
}

fn function_result(function: &Function) -> Option<BuiltinType> {
    let name = pure_name(function)?;
    if !arguments_are_proven(function) {
        return None;
    }
    match name.as_str() {
        "lower" | "upper" | "substr" | "substring" | "md5" | "replace" | "split_part" => {
            Some(BuiltinType::Text)
        }
        "length" | "char_length" | "character_length" | "octet_length" => {
            Some(BuiltinType::Integer)
        }
        "abs" => first_argument_type(&function.args).filter(|kind| kind.numeric()),
        _ => None,
    }
}

fn first_argument_type(args: &FunctionArguments) -> Option<BuiltinType> {
    let FunctionArguments::List(list) = args else {
        return None;
    };
    operand_type(function_arg_expr(list.args.first()?)?)
}
