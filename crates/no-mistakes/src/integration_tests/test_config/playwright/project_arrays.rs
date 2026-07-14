use super::{shared, Options};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::Statement::ExpressionStatement;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, FunctionBody, ObjectExpression, Program,
    Statement,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

mod calls;
mod exports;
mod function_returns;
mod functions;
mod imports;
mod member_helpers;
mod members;
mod objects;
mod root_options;
mod root_spreads;

use function_returns::body_return_options;
use functions::top_level_function_bodies;
use imports::{import_bindings, ImportBinding};
pub(super) use root_options::{root_options, root_options_from_visible};

type ExprMap<'a> = BTreeMap<String, &'a Expression<'a>>;
type FnMap<'a> = BTreeMap<String, &'a FunctionBody<'a>>;

pub(super) struct Ctx<'a, 'r> {
    source: &'a str,
    bindings: ExprMap<'a>,
    functions: FnMap<'a>,
    imports: BTreeMap<String, ImportBinding>,
    resolver: &'r ImportResolver<'r>,
    path: &'r Path,
    seen: &'r mut BTreeSet<PathBuf>,
    local_seen: &'r mut BTreeSet<String>,
    object_seen: &'r mut BTreeSet<String>,
}

pub(super) fn project_options(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    tsconfig: &TsConfig,
) -> Result<Vec<Options>> {
    project_options_inner(program, object, source, path, tsconfig, None)
}

pub(super) fn project_options_from_visible(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    tsconfig: &TsConfig,
    visible_files: &HashSet<PathBuf>,
) -> Result<Vec<Options>> {
    project_options_inner(program, object, source, path, tsconfig, Some(visible_files))
}

fn project_options_inner(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    tsconfig: &TsConfig,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<Vec<Options>> {
    let resolver = match visible_files {
        Some(visible) => ImportResolver::new(tsconfig).with_visible(visible),
        None => ImportResolver::new(tsconfig),
    };
    let mut seen = BTreeSet::new();
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: &resolver,
        path,
        seen: &mut seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    root_spreads::project_options(object, &mut ctx).map(|options| options.unwrap_or_default())
}

pub(super) fn array_options(
    projects: &ArrayExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let mut options = Vec::new();
    for element in &projects.elements {
        match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                options.extend(expression_options(&spread.argument, ctx)?);
            }
            _ => {
                if let Some(expression) = element.as_expression() {
                    if !shared::is_array_expression_reference(expression, &ctx.bindings) {
                        if let Some(option) = objects::expression_object_options(expression, ctx)? {
                            options.push(option);
                        }
                    }
                }
            }
        }
    }
    Ok(options)
}

pub(super) fn expression_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    match unwrap_ts_wrappers(expression) {
        Expression::ArrayExpression(array) => array_options(array, ctx),
        Expression::ObjectExpression(object) => {
            Ok(vec![objects::project_object_options(object, ctx)?])
        }
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

pub(in crate::integration_tests::test_config::playwright::project_arrays) fn imported_options(
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
