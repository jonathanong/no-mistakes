use super::{
    import_bindings, objects, project_options_in, shared, Ctx, ImportBinding, Options, Scope,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, FunctionBody, Statement};
use std::collections::BTreeSet;
use std::path::Path;

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
    } else if let Some(import) = ctx.imports.get(name).cloned() {
        imported_project_options(&import, ctx.path, ctx, scope)
    } else {
        Ok(None)
    };
    ctx.local_seen.remove(&key);
    result
}

fn imported_project_options(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    crate::integration_tests::test_config::shared_project_arrays::imported_function_options(
        import,
        base_path,
        ctx,
        import_bindings,
        |body, scoped| body_return_project_options(body, scoped, scope),
    )
}

fn helper_project_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow) => match arrow.body.as_expression() {
            Some(expression) => helper_project_options(expression, ctx, scope),
            None => crate::ast::arrow_function_body(&arrow.body)
                .map(|body| body_return_project_options(body, ctx, scope))
                .unwrap_or(Ok(None)),
        },
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

fn body_return_project_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let body_bindings = shared::function_body_bindings(body);
    if !body_bindings.is_empty() {
        let mut bindings = ctx.bindings.clone();
        bindings.extend(body_bindings);
        let mut local_seen = BTreeSet::new();
        let mut object_seen = BTreeSet::new();
        let mut scoped = Ctx {
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
        return body_return_project_options_without_locals(body, &mut scoped, scope);
    }
    body_return_project_options_without_locals(body, ctx, scope)
}

fn body_return_project_options_without_locals(
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
