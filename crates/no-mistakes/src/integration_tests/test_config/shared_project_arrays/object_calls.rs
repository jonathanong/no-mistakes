use super::{Context, ImportBinding};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::test_config::shared;
use anyhow::Result;
use oxc_ast::ast::{Expression, FunctionBody, Program, Statement};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

type ExpressionObjectOptions<T> = fn(&Expression<'_>, &mut Context<'_, '_>) -> Result<Option<T>>;

pub(in crate::integration_tests::test_config) struct ObjectCallOps<T> {
    pub expression_object_options: ExpressionObjectOptions<T>,
    pub import_bindings: fn(&Program<'_>) -> BTreeMap<String, ImportBinding>,
}

pub(in crate::integration_tests::test_config) fn call_object_options<T>(
    callee: &Expression<'_>,
    ctx: &mut Context<'_, '_>,
    ops: &ObjectCallOps<T>,
) -> Result<Option<T>> {
    let Expression::Identifier(identifier) = callee else {
        return Ok(None);
    };
    let name = identifier.name.as_str();
    let key = format!("object-call:{name}");
    if !ctx.local_seen.insert(key.clone()) {
        return Ok(None);
    }
    let result = if let Some(expression) = ctx.bindings.get(name).copied() {
        helper_object_options(expression, ctx, ops)
    } else if let Some(body) = ctx.functions.get(name).copied() {
        body_return_object_options(body, ctx, ops)
    } else if let Some(import) = ctx.imports.get(name).cloned() {
        imported_object_options(&import, ctx.path, ctx, ops)
    } else {
        Ok(None)
    };
    ctx.local_seen.remove(&key);
    result
}

fn imported_object_options<T>(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Context<'_, '_>,
    ops: &ObjectCallOps<T>,
) -> Result<Option<T>> {
    crate::integration_tests::test_config::shared_project_arrays::imported_function_options(
        import,
        base_path,
        ctx,
        ops.import_bindings,
        |body, ctx| body_return_object_options(body, ctx, ops),
    )
}

fn helper_object_options<T>(
    expression: &Expression<'_>,
    ctx: &mut Context<'_, '_>,
    ops: &ObjectCallOps<T>,
) -> Result<Option<T>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow) => match arrow.body.as_expression() {
            Some(expression) => (ops.expression_object_options)(expression, ctx),
            None => crate::ast::arrow_function_body(&arrow.body)
                .map(|body| body_return_object_options(body, ctx, ops))
                .unwrap_or(Ok(None)),
        },
        Expression::FunctionExpression(function) => match function.body.as_deref() {
            Some(body) => body_return_object_options(body, ctx, ops),
            None => Ok(None),
        },
        _ => (ops.expression_object_options)(expression, ctx),
    }
}

fn body_return_object_options<T>(
    body: &FunctionBody<'_>,
    ctx: &mut Context<'_, '_>,
    ops: &ObjectCallOps<T>,
) -> Result<Option<T>> {
    let body_bindings = shared::function_body_bindings(body);
    if !body_bindings.is_empty() {
        let mut bindings = ctx.bindings.clone();
        bindings.extend(body_bindings);
        let mut local_seen = BTreeSet::new();
        let mut object_seen = BTreeSet::new();
        let mut scoped = Context {
            is_test_object: ctx.is_test_object,
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
        return body_return_object_options_without_locals(body, &mut scoped, ops);
    }
    body_return_object_options_without_locals(body, ctx, ops)
}

fn body_return_object_options_without_locals<T>(
    body: &FunctionBody<'_>,
    ctx: &mut Context<'_, '_>,
    ops: &ObjectCallOps<T>,
) -> Result<Option<T>> {
    for statement in &body.statements {
        let Statement::ReturnStatement(return_statement) = statement else {
            continue;
        };
        let Some(argument) = &return_statement.argument else {
            continue;
        };
        return (ops.expression_object_options)(argument, ctx);
    }
    Ok(None)
}
