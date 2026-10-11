use super::{public_config_call, Ctx, Expression, Program, Statement};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrowFunctionBody, FunctionBody, ObjectExpression};

mod exports;
mod spreads;

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn callback_config_object<
    'a,
>(
    program: &'a Program<'a>,
    ctx: &Ctx<'a, '_>,
) -> Option<&'a ObjectExpression<'a>> {
    let bindings = exports::immutable_bindings(program);
    if let Some(expression) = exports::default_expression(program, &bindings) {
        let expression = super::super::super::shared::expression_value(expression, &bindings);
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        if call.optional || call.arguments.len() != 1 {
            return None;
        }
        let role = public_config_call(&call.callee, program, ctx)?;
        if !matches!(role.as_str(), "defineConfig" | "defineProject") {
            return None;
        }
        let object = callback_object(call.arguments[0].as_expression()?)?;
        return spreads::proven_object(object).then_some(object);
    }
    None
}

fn callback_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow)
            if arrow.params.items.is_empty() && arrow.params.rest.is_none() =>
        {
            match &arrow.body {
                ArrowFunctionBody::FunctionBody(body) => body_object(body),
                body => literal_object(body.as_expression()?),
            }
        }
        Expression::FunctionExpression(function)
            if !function.generator
                && function.params.items.is_empty()
                && function.params.rest.is_none() =>
        {
            body_object(function.body.as_ref()?)
        }
        _ => None,
    }
}

fn body_object<'a>(body: &'a FunctionBody<'a>) -> Option<&'a ObjectExpression<'a>> {
    let [statement] = body.statements.as_slice() else {
        return None;
    };
    let Statement::ReturnStatement(statement) = statement else {
        return None;
    };
    literal_object(statement.argument.as_ref()?)
}

fn literal_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}
