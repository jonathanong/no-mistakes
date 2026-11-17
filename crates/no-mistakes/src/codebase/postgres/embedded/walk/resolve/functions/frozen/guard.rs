use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{self, BinaryOperator, Expression, LogicalOperator, Statement, UnaryOperator};

fn has_own<'a>(test: &'a Expression<'a>) -> Option<(&'a str, &'a str)> {
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

pub(super) fn guarded_record<'a>(body: &'a ast::FunctionBody<'a>) -> Option<(&'a str, &'a str)> {
    guarded_record_with_span(body).map(|(record, key, _)| (record, key))
}

pub(super) fn guarded_record_with_span<'a>(
    body: &'a ast::FunctionBody<'a>,
) -> Option<(&'a str, &'a str, u32)> {
    let (last, guards) = body.statements.as_slice().split_last()?;
    let ast::Statement::ReturnStatement(returned) = last else {
        return None;
    };
    if guards.is_empty() {
        return None;
    }
    let (record, key, start) = returned_record(returned)?;
    let mut saw_own = false;
    for statement in guards {
        let ast::Statement::IfStatement(guard) = statement else {
            return None;
        };
        if guard.alternate.is_some() || !stops(&guard.consequent) {
            return None;
        }
        if !guard_mentions(&guard.test, record, key, &mut saw_own) {
            return None;
        }
    }
    saw_own.then_some((record, key, start))
}

fn guard_mentions(test: &Expression<'_>, record: &str, key: &str, saw_own: &mut bool) -> bool {
    let test = unwrap_ts_wrappers(test);
    if let Expression::LogicalExpression(logic) = test {
        if logic.operator != LogicalOperator::Or {
            return false;
        }
        return guard_mentions(&logic.left, record, key, saw_own)
            && guard_mentions(&logic.right, record, key, saw_own);
    }
    if let Some((own_record, own_key)) = has_own(test) {
        if own_record == record && own_key == key {
            *saw_own = true;
            return true;
        }
        return false;
    }
    typeof_string(test, key)
}

fn typeof_string(test: &Expression<'_>, key: &str) -> bool {
    let Expression::BinaryExpression(binary) = unwrap_ts_wrappers(test) else {
        return false;
    };
    if !matches!(
        binary.operator,
        BinaryOperator::StrictInequality | BinaryOperator::Inequality
    ) {
        return false;
    }
    let Expression::UnaryExpression(unary) = unwrap_ts_wrappers(&binary.left) else {
        return false;
    };
    if unary.operator != UnaryOperator::Typeof {
        return false;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(&unary.argument) else {
        return false;
    };
    if id.name != key {
        return false;
    }
    let Expression::StringLiteral(literal) = unwrap_ts_wrappers(&binary.right) else {
        return false;
    };
    literal.value == "string"
}

fn stops(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ThrowStatement(_) => true,
        Statement::ReturnStatement(returned) => early_return(returned),
        Statement::BlockStatement(block) => match block.body.as_slice() {
            [Statement::ThrowStatement(_)] => true,
            [Statement::ReturnStatement(returned)] => early_return(returned),
            _ => false,
        },
        _ => false,
    }
}

fn early_return(returned: &ast::ReturnStatement<'_>) -> bool {
    let Some(argument) = returned.argument.as_ref() else {
        return true;
    };
    match unwrap_ts_wrappers(argument) {
        Expression::NullLiteral(_) => true,
        Expression::Identifier(id) => id.name == "undefined",
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::Void,
        _ => false,
    }
}

fn returned_record<'a>(returned: &'a ast::ReturnStatement<'a>) -> Option<(&'a str, &'a str, u32)> {
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
    Some((object.name.as_str(), key.name.as_str(), object.span.start))
}
