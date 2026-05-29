use super::{objects, project_options_in, Ctx, Options, Scope};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, FunctionBody, Statement};

pub(super) fn call_project_options(
    callee: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let Expression::Identifier(identifier) = callee else {
        return Ok(None);
    };
    let name = identifier.name.as_str();
    let key = format!("root-call:{name}");
    if !ctx.local_seen.insert(key.clone()) {
        return Ok(None);
    }
    let result = if let Some(expression) = ctx.bindings.get(name).copied() {
        helper_project_options(expression, ctx, scope)
    } else if let Some(body) = ctx.functions.get(name).copied() {
        body_return_project_options(body, ctx, scope)
    } else {
        Ok(None)
    };
    ctx.local_seen.remove(&key);
    result
}

fn helper_project_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            expression_body_project_options(&arrow.body, ctx, scope)
        }
        Expression::ArrowFunctionExpression(arrow) => {
            body_return_project_options(&arrow.body, ctx, scope)
        }
        Expression::FunctionExpression(function) => match function.body.as_deref() {
            Some(body) => body_return_project_options(body, ctx, scope),
            None => Ok(None),
        },
        _ => {
            let Some(object) = objects::expression_object(expression, &ctx.bindings) else {
                return Ok(None);
            };
            project_options_in(object, ctx, scope)
        }
    }
}

fn expression_body_project_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let Some(Statement::ExpressionStatement(statement)) = body.statements.first() else {
        return Ok(None);
    };
    helper_project_options(&statement.expression, ctx, scope)
}

fn body_return_project_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    for statement in &body.statements {
        let Statement::ReturnStatement(return_statement) = statement else {
            continue;
        };
        let Some(argument) = &return_statement.argument else {
            continue;
        };
        return helper_project_options(argument, ctx, scope);
    }
    Ok(None)
}
