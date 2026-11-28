use super::super::function::{
    arguments_are_proven, bare_arguments, positional_argument, pure_name,
};
use super::{operand_type, substring_is_proven, BuiltinType};
use sqlparser::ast::Function;

pub(super) fn function_result(function: &Function) -> Option<BuiltinType> {
    let name = pure_name(function)?;
    match name.as_str() {
        "abs" => {
            let [argument] = bare_arguments(function)? else {
                return None;
            };
            operand_type(positional_argument(argument)?).filter(|kind| kind.numeric())
        }
        "substr" | "substring" => substring_result(function, &name),
        "lower" | "upper" => {
            let [argument] = bare_arguments(function)? else {
                return None;
            };
            (operand_type(positional_argument(argument)?) == Some(BuiltinType::Text))
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
    let source = positional_argument(source)?;
    let start = positional_argument(start)?;
    let length = match tail {
        [] => None,
        [length] => Some(positional_argument(length)?),
        _ => return None,
    };
    substring_is_proven(source, Some(start), length, name == "substring")
        .then_some(BuiltinType::Text)
}
