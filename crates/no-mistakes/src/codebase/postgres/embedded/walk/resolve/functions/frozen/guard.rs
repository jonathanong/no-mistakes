use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{self, Expression, Statement, UnaryOperator};

pub(super) fn has_own<'a>(test: &'a Expression<'a>) -> Option<(&'a str, &'a str)> {
    let Expression::UnaryExpression(unary) = unwrap_ts_wrappers(test) else {
        return None;
    };
    if unary.operator != UnaryOperator::LogicalNot {
        return None;
    }
    let Expression::CallExpression(call) = unwrap_ts_wrappers(&unary.argument) else {
        return None;
    };
    if call.optional || call.arguments.len() != 2 || !object_has_own(&call.callee) {
        return None;
    }
    Some((
        ident_arg(&call.arguments[0])?,
        ident_arg(&call.arguments[1])?,
    ))
}

fn object_has_own(callee: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(callee) else {
        return false;
    };
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return false;
    };
    !member.optional && object.name == "Object" && member.property.name == "hasOwn"
}

fn ident_arg<'a>(argument: &'a ast::Argument<'a>) -> Option<&'a str> {
    let Expression::Identifier(id) = unwrap_ts_wrappers(argument.as_expression()?) else {
        return None;
    };
    Some(id.name.as_str())
}

pub(super) fn throws(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => {
            matches!(block.body.as_slice(), [Statement::ThrowStatement(_)])
        }
        _ => false,
    }
}

pub(super) fn returned_record<'a>(
    returned: &'a ast::ReturnStatement<'a>,
) -> Option<(&'a str, &'a str)> {
    let Expression::ComputedMemberExpression(member) =
        unwrap_ts_wrappers(returned.argument.as_ref()?)
    else {
        return None;
    };
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let Expression::Identifier(key) = unwrap_ts_wrappers(&member.expression) else {
        return None;
    };
    Some((object.name.as_str(), key.name.as_str()))
}
