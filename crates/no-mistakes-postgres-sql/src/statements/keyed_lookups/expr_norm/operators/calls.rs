use super::super::function::{arguments_are_proven, bare_arguments, pure_name};
use super::{operand_type, substring_is_proven, BuiltinType};
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr};

pub(super) fn function_result(function: &Function) -> Option<BuiltinType> {
    let name = pure_name(function)?;
    match name.as_str() {
        "abs" => {
            let [argument] = bare_arguments(function)? else {
                return None;
            };
            operand_type(positional_expr(argument)?).filter(|kind| kind.numeric())
        }
        "substr" | "substring" => substring_result(function, &name),
        "lower" | "upper" => {
            let [argument] = bare_arguments(function)? else {
                return None;
            };
            (operand_type(positional_expr(argument)?) == Some(BuiltinType::Text))
                .then_some(BuiltinType::Text)
        }
        "md5" | "replace" | "split_part" if arguments_are_proven(function) => {
            Some(BuiltinType::Text)
        }
        "length" | "char_length" | "character_length" | "octet_length"
            if arguments_are_proven(function) =>
        {
            Some(BuiltinType::Integer)
        }
        _ => None,
    }
}

fn substring_result(function: &Function, name: &str) -> Option<BuiltinType> {
    let [source, start, tail @ ..] = bare_arguments(function)? else {
        return None;
    };
    let source = positional_expr(source)?;
    let start = positional_expr(start)?;
    let length = match tail {
        [] => None,
        [length] => Some(positional_expr(length)?),
        _ => return None,
    };
    if name == "substr" && operand_type(start) != Some(BuiltinType::Integer) {
        return None;
    }
    substring_is_proven(source, Some(start), length).then_some(BuiltinType::Text)
}

fn positional_expr(arg: &FunctionArg) -> Option<&Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => Some(expr),
        _ => None,
    }
}
