use super::{parse_partial_options, shared, Options};
use crate::ast;
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, FunctionBody, ObjectExpression, Program,
    Statement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod calls;
mod exports;
mod imports;

use imports::{import_bindings, ImportBinding};

type ExprMap<'a> = BTreeMap<String, &'a Expression<'a>>;
type FnMap<'a> = BTreeMap<String, &'a FunctionBody<'a>>;

pub(super) struct Ctx<'a, 'r> {
    source: &'a str,
    bindings: ExprMap<'a>,
    functions: FnMap<'a>,
    imports: BTreeMap<String, ImportBinding>,
    resolver: &'r ImportResolver<'r>,
    path: &'r Path,
    root: &'r Path,
    seen: &'r mut BTreeSet<PathBuf>,
    local_seen: &'r mut BTreeSet<String>,
}

pub(super) fn project_options(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    root: &Path,
    tsconfig: &TsConfig,
) -> Vec<Options> {
    let Some(Expression::ArrayExpression(projects)) =
        shared::property_expression(object, "projects")
    else {
        return Vec::new();
    };
    let resolver = ImportResolver::new(tsconfig);
    let mut seen = BTreeSet::new();
    let mut local_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: &resolver,
        path,
        root,
        seen: &mut seen,
        local_seen: &mut local_seen,
    };
    array_options(projects, &mut ctx)
}

pub(super) fn array_options(projects: &ArrayExpression<'_>, ctx: &mut Ctx<'_, '_>) -> Vec<Options> {
    let mut options = Vec::new();
    for element in &projects.elements {
        match element {
            ArrayExpressionElement::ObjectExpression(object) => {
                options.extend(project_object_options(object, ctx));
            }
            ArrayExpressionElement::SpreadElement(spread) => {
                options.extend(expression_options(&spread.argument, ctx));
            }
            _ => {}
        }
    }
    options
}

fn project_object_options(object: &ObjectExpression<'_>, ctx: &Ctx<'_, '_>) -> Option<Options> {
    let nested = shared::property_object(object, "test", &ctx.bindings).unwrap_or(object);
    Some(parse_partial_options(nested, ctx.source))
}

pub(super) fn expression_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<Options> {
    match expression {
        Expression::ArrayExpression(array) => array_options(array, ctx),
        Expression::Identifier(identifier) => identifier_options(identifier.name.as_str(), ctx),
        Expression::CallExpression(call) => calls::call_options(&call.callee, ctx),
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_options(&parenthesized.expression, ctx)
        }
        _ => Vec::new(),
    }
}

fn identifier_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Vec<Options> {
    if !ctx.local_seen.insert(name.to_string()) {
        return Vec::new();
    }
    let result = ctx
        .bindings
        .get(name)
        .copied()
        .map(|expression| expression_options(expression, ctx))
        .or_else(|| {
            ctx.imports
                .get(name)
                .cloned()
                .map(|import| imported_options(&import, ctx))
        })
        .unwrap_or_default();
    ctx.local_seen.remove(name);
    result
}

pub(super) fn helper_expression_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<Options> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            expression_statement_options(&arrow.body, ctx)
        }
        Expression::ArrowFunctionExpression(arrow) => body_return_options(&arrow.body, ctx),
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()
            .map(|body| body_return_options(body, ctx))
            .unwrap_or_default(),
        _ => expression_options(expression, ctx),
    }
}

pub(super) fn body_return_options(body: &FunctionBody<'_>, ctx: &mut Ctx<'_, '_>) -> Vec<Options> {
    for statement in &body.statements {
        let Statement::ReturnStatement(return_statement) = statement else {
            continue;
        };
        if let Some(argument) = &return_statement.argument {
            return expression_options(argument, ctx);
        }
    }
    Vec::new()
}

pub(super) fn expression_statement_options(
    body: &FunctionBody<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<Options> {
    let Some(Statement::ExpressionStatement(statement)) = body.statements.first() else {
        return Vec::new();
    };
    expression_options(&statement.expression, ctx)
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn imported_options(
    import: &ImportBinding,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<Options> {
    let Some(path) = ctx.resolver.resolve(&import.source, ctx.path) else {
        return Vec::new();
    };
    if !ctx.seen.insert(path.clone()) {
        return Vec::new();
    }
    let result = std::fs::read_to_string(&path)
        .ok()
        .and_then(|source| {
            ast::with_program(&path, &source, |program, source| {
                exports::exported_options(program, source, &path, ctx, &import.imported)
            })
            .ok()
        })
        .unwrap_or_default();
    ctx.seen.remove(&path);
    result
}

pub(super) fn top_level_function_bodies<'a>(program: &'a Program<'a>) -> FnMap<'a> {
    program
        .body
        .iter()
        .filter_map(|statement| {
            let Statement::FunctionDeclaration(function) = statement else {
                return None;
            };
            Some((
                function.id.as_ref()?.name.to_string(),
                function.body.as_ref()?.as_ref(),
            ))
        })
        .collect()
}
