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
    _arguments: &[Expression],
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
    }
}
