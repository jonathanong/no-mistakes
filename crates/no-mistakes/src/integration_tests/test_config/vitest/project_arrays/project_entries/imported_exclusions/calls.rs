use super::super::super::{shared, Ctx};
use super::{extend_expression_exclusions, extend_imported_call_exclusions};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, FunctionBody, Statement};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(super) fn extend_call_exclusions(
    callee: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
    excluded: &mut BTreeSet<PathBuf>,
) {
    let Expression::Identifier(identifier) = unwrap_ts_wrappers(callee) else {
        return;
    };
    let name = identifier.name.as_str();
    let key = format!("call:{name}");
    if !ctx.local_seen.insert(key.clone()) {
        return;
    }
    if let Some(expression) = ctx.bindings.get(name).copied() {
        extend_callable_exclusions(expression, ctx, excluded);
    } else if let Some(body) = ctx.functions.get(name).copied() {
        extend_function_body_exclusions(body, ctx, excluded);
    } else if let Some(import) = ctx.imports.get(name).cloned() {
        extend_imported_call_exclusions(&import, ctx, excluded);
    }
    ctx.local_seen.remove(&key);
}

pub(super) fn extend_callable_exclusions(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
    excluded: &mut BTreeSet<PathBuf>,
) {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            let Some(Statement::ExpressionStatement(statement)) = arrow.body.statements.first()
            else {
                return;
            };
            extend_expression_exclusions(&statement.expression, ctx, excluded);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            extend_function_body_exclusions(&arrow.body, ctx, excluded);
        }
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                extend_function_body_exclusions(body, ctx, excluded);
            }
        }
        _ => {}
    }
}

pub(super) fn extend_function_body_exclusions(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
    excluded: &mut BTreeSet<PathBuf>,
) {
    let body_bindings = shared::function_body_bindings(body);
    if body_bindings.is_empty() {
        return extend_return_exclusions(body, ctx, excluded);
    }
    let mut bindings = ctx.bindings.clone();
    bindings.extend(body_bindings);
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut scoped = Ctx {
        source: ctx.source,
        bindings,
        functions: ctx.functions.clone(),
        imports: ctx.imports.clone(),
        resolver: ctx.resolver,
        path: ctx.path,
        seen: ctx.seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    extend_return_exclusions(body, &mut scoped, excluded);
}

fn extend_return_exclusions(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
    excluded: &mut BTreeSet<PathBuf>,
) {
    let Some(argument) = body
        .statements
        .iter()
        .find_map(|statement| match statement {
            Statement::ReturnStatement(return_statement) => return_statement.argument.as_ref(),
            _ => None,
        })
    else {
        return;
    };
    extend_expression_exclusions(argument, ctx, excluded);
}
