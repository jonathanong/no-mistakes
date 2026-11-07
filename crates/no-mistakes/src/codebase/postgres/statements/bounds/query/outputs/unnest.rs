use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use sqlparser::ast::{
    ArrayElemTypeDef, DataType, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments,
    ObjectNamePart,
};

pub(super) fn scalar(function: &Function) -> bool {
    let builtin = match function.name.0.as_slice() {
        [name] => named(name, "unnest"),
        [schema, name] => named(schema, "pg_catalog") && named(name, "unnest"),
        _ => false,
    };
    if !builtin
        || function.over.is_some()
        || function.filter.is_some()
        || !function.within_group.is_empty()
        || !matches!(function.parameters, FunctionArguments::None)
    {
        return false;
    }
    let FunctionArguments::List(arguments) = &function.args else {
        return false;
    };
    if !arguments.clauses.is_empty() || arguments.duplicate_treatment.is_some() {
        return false;
    }
    let [FunctionArg::Unnamed(FunctionArgExpr::Expr(argument))] = arguments.args.as_slice() else {
        return false;
    };
    let Expr::Cast {
        data_type: DataType::Array(element),
        ..
    } = unwrap_expr(argument)
    else {
        return false;
    };
    scalar_element(element)
}

fn named(part: &ObjectNamePart, expected: &str) -> bool {
    part.as_ident()
        .is_some_and(|ident| ident_key(ident) == expected)
}

fn scalar_element(element: &ArrayElemTypeDef) -> bool {
    let data_type = match element {
        ArrayElemTypeDef::AngleBracket(inner)
        | ArrayElemTypeDef::SquareBracket(inner, _)
        | ArrayElemTypeDef::Parenthesis(inner)
        | ArrayElemTypeDef::Qualified(inner, _) => inner.as_ref(),
        _ => return false,
    };
    match data_type {
        DataType::Array(inner) => scalar_element(inner),
        DataType::Uuid
        | DataType::Text
        | DataType::Boolean
        | DataType::Bool
        | DataType::Int(_)
        | DataType::Integer(_)
        | DataType::BigInt(_)
        | DataType::SmallInt(_)
        | DataType::Numeric(_)
        | DataType::Decimal(_) => true,
        _ => false,
    }
}
