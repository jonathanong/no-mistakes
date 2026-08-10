use super::Expression;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    lexer::Function, StaticExpressionType,
};

pub(super) fn logical_result(left: Expression, right: Expression) -> Expression {
    if left.static_type == right.static_type && left.static_type != StaticExpressionType::Dynamic {
        Expression::scalar(left.static_type)
    } else {
        Expression::dynamic(left.may_produce_mapping || right.may_produce_mapping)
    }
}

pub(super) fn function_static_type(
    function: Function,
    arguments: &[Expression],
) -> StaticExpressionType {
    match function {
        Function::Contains
        | Function::StartsWith
        | Function::EndsWith
        | Function::Success
        | Function::Failure
        | Function::Always
        | Function::Cancelled => StaticExpressionType::Boolean,
        Function::Format | Function::Join | Function::ToJson | Function::HashFiles => {
            StaticExpressionType::String
        }
        Function::FromJson => StaticExpressionType::Dynamic,
        Function::Case => case_static_type(arguments),
    }
}

fn case_static_type(arguments: &[Expression]) -> StaticExpressionType {
    let mut branches = arguments
        .iter()
        .enumerate()
        .filter_map(|(index, argument)| {
            (index % 2 == 1 || index + 1 == arguments.len()).then_some(argument.static_type)
        });
    let Some(static_type) = branches.next() else {
        return StaticExpressionType::Dynamic;
    };
    if branches.all(|branch| branch == static_type && branch != StaticExpressionType::Dynamic) {
        static_type
    } else {
        StaticExpressionType::Dynamic
    }
}
