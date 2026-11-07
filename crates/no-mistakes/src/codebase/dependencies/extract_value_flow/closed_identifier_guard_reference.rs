use crate::codebase::dependencies::extract::ValueExpression;
use oxc_ast::ast::{Expression, Statement};
use oxc_syntax::operator::UnaryOperator;

pub(super) fn value(value: &ValueExpression) -> Option<(String, usize, Vec<String>)> {
    match value {
        ValueExpression::Named {
            name,
            scope: Some(scope),
            ..
        } => Some((name.clone(), *scope, Vec::new())),
        ValueExpression::Member(base, key) => {
            let (name, scope, mut members) = self::value(base)?;
            members.push(key.clone());
            Some((name, scope, members))
        }
        _ => None,
    }
}

pub(super) fn expression(expr: &Expression<'_>) -> Option<()> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::Identifier(_) => Some(()),
        Expression::StaticMemberExpression(member) if !member.optional => {
            expression(&member.object)
        }
        Expression::ComputedMemberExpression(member) if !member.optional => {
            super::super::literal_string(&member.expression)?;
            expression(&member.object)
        }
        _ => None,
    }
}

pub(super) fn closed_set_guard<'a>(
    statement: &'a Statement<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    let Statement::IfStatement(branch) = statement else {
        return None;
    };
    if branch.alternate.is_some()
        || super::super::super::completion::statement_completes(&branch.consequent)
    {
        return None;
    }
    let Expression::UnaryExpression(unary) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&branch.test)
    else {
        return None;
    };
    if unary.operator != UnaryOperator::LogicalNot {
        return None;
    }
    let call = super::super::plain_call(&unary.argument)?;
    if call.arguments.len() != 1 {
        return None;
    }
    expression(call.arguments[0].as_expression()?)?;
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    match member.property.name.as_str() {
        "has" | "includes" => {}
        _ => return None,
    }
    Some(call)
}

pub(super) fn identifier_name(expr: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}
