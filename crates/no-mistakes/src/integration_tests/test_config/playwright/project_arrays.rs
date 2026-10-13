use super::{shared, Options};
use crate::codebase::ts_resolver::ImportResolution;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, ArrowFunctionBody, Expression, FunctionBody,
    ObjectExpression, Program, Statement,
};
use oxc_span::GetSpan;
use std::collections::BTreeSet;
use std::path::Path;

mod calls {
    use super::{body_return_options, helper_expression_options, imported_options, Ctx, Options};
    use anyhow::Result;
    use oxc_ast::ast::Expression;

    pub(super) fn call_options(
        callee: &Expression<'_>,
        ctx: &mut Ctx<'_, '_>,
    ) -> Result<Vec<Options>> {
        crate::integration_tests::test_config::shared_project_arrays::calls::call_options(
            callee,
            ctx,
            helper_expression_options,
            body_return_options,
            imported_options,
        )
    }
}
mod exports;
mod function_returns;
mod imports;
use crate::integration_tests::test_config::shared_project_arrays::members::{self, MemberOps};
mod objects;
mod root_options;
mod root_spreads;

use crate::integration_tests::test_config::shared_project_arrays::top_level_function_bodies;
use function_returns::body_return_options;
use imports::{import_bindings, ImportBinding};
pub(super) use root_options::root_options;

use crate::integration_tests::test_config::shared_project_arrays::{Context as Ctx, ExprMap};

fn member_ops() -> MemberOps<Options> {
    MemberOps {
        expression_options,
        imported_options,
        exported_options: exports::exported_options,
        expression_object: objects::expression_object,
        import_bindings,
        default_identifier_reexport: false,
    }
}

pub(super) fn project_options(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<Options>> {
    project_options_inner(program, object, source, path, resolver)
}

fn project_options_inner(
    program: &Program<'_>,
    object: &ObjectExpression<'_>,
    source: &str,
    path: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<Options>> {
    let mut seen = BTreeSet::new();
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        is_test_object: false,
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver,
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
    let mut options = match unwrap_ts_wrappers(expression) {
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
            members::namespace_member_options(member, ctx, &member_ops())
        }
        _ => Ok(Vec::new()),
    }?;
    if !matches!(
        unwrap_ts_wrappers(expression),
        Expression::ObjectExpression(_) | Expression::ArrayExpression(_)
    ) {
        for option in &mut options {
            crate::integration_tests::test_config::deadlines::unproved_case(
                &mut option.deadlines,
                ctx.path,
                expression.span(),
            );
        }
    }
    Ok(options)
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
        Expression::ArrowFunctionExpression(arrow) => match &arrow.body {
            ArrowFunctionBody::FunctionBody(body) => body_return_options(body, ctx),
            _ => expression_options(arrow.body.to_expression(), ctx),
        },
        Expression::FunctionExpression(function) => match function.body.as_deref() {
            Some(body) => body_return_options(body, ctx),
            None => Ok(Vec::new()),
        },
        _ => expression_options(expression, ctx),
    }
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
