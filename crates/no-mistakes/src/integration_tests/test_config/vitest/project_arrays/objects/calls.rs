use super::super::{
    import_bindings, shared, top_level_function_bodies, Ctx, ImportBinding, Options,
};
use super::expression_object_options;
use crate::ast;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, FunctionBody, Statement};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn call_object_options(
    callee: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Expression::Identifier(identifier) = callee else {
        return Ok(None);
    };
    let name = identifier.name.as_str();
    let key = format!("object-call:{name}");
    if !ctx.local_seen.insert(key.clone()) {
        return Ok(None);
    }
    let result = if let Some(expression) = ctx.bindings.get(name).copied() {
        helper_object_options(expression, ctx)
    } else if let Some(body) = ctx.functions.get(name).copied() {
        body_return_object_options(body, ctx)
    } else if let Some(import) = ctx.imports.get(name).cloned() {
        imported_object_options(&import, ctx.path, ctx)
    } else {
        Ok(None)
    };
    ctx.local_seen.remove(&key);
    result
}

fn imported_object_options(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(None);
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(None);
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(None),
        Ok(source) => ast::with_program(&path, &source, |program, source| {
            let bindings = shared::top_level_object_bindings(program);
            let functions = top_level_function_bodies(program);
            let Some(body) = functions.get(import.imported.as_str()).copied() else {
                return Ok(None);
            };
            let mut local_seen = BTreeSet::new();
            let mut object_seen = BTreeSet::new();
            let mut scoped = Ctx {
                source,
                bindings,
                functions,
                imports: import_bindings(program),
                resolver: ctx.resolver,
                path: &path,
                seen: ctx.seen,
                local_seen: &mut local_seen,
                object_seen: &mut object_seen,
            };
            body_return_object_options(body, &mut scoped)
        })
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn helper_object_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            expression_body_object_options(&arrow.body, ctx)
        }
        Expression::ArrowFunctionExpression(arrow) => body_return_object_options(&arrow.body, ctx),
        Expression::FunctionExpression(function) => match function.body.as_deref() {
            Some(body) => body_return_object_options(body, ctx),
            None => Ok(None),
        },
        _ => expression_object_options(expression, ctx),
    }
}

fn expression_body_object_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Some(Statement::ExpressionStatement(statement)) = body.statements.first() else {
        return Ok(None);
    };
    expression_object_options(&statement.expression, ctx)
}

fn body_return_object_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let body_bindings = shared::function_body_bindings(body);
    if !body_bindings.is_empty() {
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
        return body_return_object_options_without_locals(body, &mut scoped);
    }
    body_return_object_options_without_locals(body, ctx)
}

fn body_return_object_options_without_locals(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    for statement in &body.statements {
        let Statement::ReturnStatement(return_statement) = statement else {
            continue;
        };
        let Some(argument) = &return_statement.argument else {
            continue;
        };
        return expression_object_options(argument, ctx);
    }
    Ok(None)
}
