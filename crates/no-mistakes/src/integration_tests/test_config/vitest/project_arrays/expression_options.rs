use super::{
    array_options, body_return_options, calls, exports, members, objects, Ctx, ImportBinding,
    Options,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::Statement::ExpressionStatement;
use oxc_ast::ast::{Expression, FunctionBody};

pub(super) fn expression_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrayExpression(array) => array_options(array, ctx),
        Expression::ObjectExpression(object) => Ok(vec![objects::project_options(object, ctx)?]),
        Expression::Identifier(identifier) => identifier_options(identifier.name.as_str(), ctx),
        Expression::CallExpression(call) if call.arguments.is_empty() => {
            calls::call_options(&call.callee, ctx)
        }
        Expression::CallExpression(_) => Ok(Vec::new()),
        Expression::StaticMemberExpression(member) => {
            members::namespace_member_options(member, ctx)
        }
        _ => Ok(Vec::new()),
    }
}

fn identifier_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Result<Vec<Options>> {
    if !ctx.local_seen.insert(name.to_string()) {
        return Ok(Vec::new());
    }
    let result = if let Some(expression) = ctx.bindings.get(name).copied() {
        expression_options(expression, ctx)
    } else if let Some(import) = ctx.imports.get(name).cloned() {
        imported_options(&import, ctx)
    } else {
        Ok(Vec::new())
    };
    ctx.local_seen.remove(name);
    result
}

pub(super) fn helper_expression_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let expression = unwrap_ts_wrappers(expression);
    match expression {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            expression_statement_options(&arrow.body, ctx)
        }
        Expression::ArrowFunctionExpression(arrow) => body_return_options(&arrow.body, ctx),
        Expression::FunctionExpression(function) => match function.body.as_deref() {
            Some(body) => body_return_options(body, ctx),
            None => Ok(Vec::new()),
        },
        _ => expression_options(expression, ctx),
    }
}

#[rustfmt::skip]
pub(super) fn expression_statement_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Some(ExpressionStatement(statement)) = body.statements.first() else { return Ok(Vec::new()) };
    expression_options(&statement.expression, ctx)
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn imported_options(
    import: &ImportBinding,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, ctx.path) else {
        return Ok(Vec::new());
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(Vec::new());
    }
    let result = match crate::integration_tests::runner_config::read_request_source(&path) {
        Err(_) => Ok(Vec::new()),
        Ok(source) => crate::integration_tests::runner_config::with_program(
            &path,
            &source,
            |program, source| {
                exports::exported_options(program, source, &path, ctx, &import.imported)
            },
        )
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}
