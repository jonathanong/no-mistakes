//! Scalar array element layouts are known only from explicit builtin types or scalar literals.
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{ArrayElemTypeDef, DataType, Expr, Value};

pub(super) fn scalar_array(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Cast {
            data_type: DataType::Array(array),
            ..
        } => scalar_type(array),
        Expr::Array(array) => array.elem.iter().all(|expr| match unwrap_expr(expr) {
            Expr::Value(value) => !matches!(value.value, Value::Placeholder(_)),
            Expr::Array(_) => scalar_array(expr),
            _ => false,
        }),
        _ => false,
    }
}
fn scalar_type(array: &ArrayElemTypeDef) -> bool {
    match array {
        ArrayElemTypeDef::SquareBracket(data_type, _)
        | ArrayElemTypeDef::Qualified(data_type, _) => match &**data_type {
            DataType::Array(array) => scalar_type(array),
            DataType::Custom(_, _) | DataType::Struct(_, _) | DataType::Tuple(_) => false,
            _ => true,
        },
        _ => false,
    }
}
