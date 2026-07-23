use super::super::{expression_object, object_expressions, project_options, shared, Ctx, Options};
use anyhow::Result;
use oxc_ast::ast::{CallExpression, ExportDefaultDeclarationKind, Expression, Program, Statement};

/// Parse only static `mergeConfig` arguments. Unknown arguments make the
/// inherited config conservative rather than accepting a partial merge.
pub(super) fn default_options(
    program: &Program<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Some(Statement::ExportDefaultDeclaration(export)) = program
        .body
        .iter()
        .find(|statement| matches!(statement, Statement::ExportDefaultDeclaration(_)))
    else {
        return default_object_options(program, ctx);
    };
    let ExportDefaultDeclarationKind::CallExpression(call) = &export.declaration else {
        return default_object_options(program, ctx);
    };
    if !is_merge_config(call) {
        return default_object_options(program, ctx);
    }
    let mut merged = Options::default();
    for argument in &call.arguments {
        let Some(expression) = argument.as_expression() else {
            return Ok(None);
        };
        let Some(options) = argument_options(expression, ctx)? else {
            return Ok(None);
        };
        merge_options(&mut merged, options);
    }
    Ok((!call.arguments.is_empty()).then_some(merged))
}

fn merge_options(base: &mut Options, next: Options) {
    let inherited_setup = base.setup_files.take();
    let inherited_global = base.global_setup.take();
    let next_setup = next.setup_files.clone();
    let next_global = next.global_setup.clone();
    let nested = next.nested_test_scope;
    super::super::merge::merge_options(base, next);
    if nested {
        base.setup_files =
            crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
                inherited_setup,
                next_setup,
            );
        base.global_setup =
            crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
                inherited_global,
                next_global,
            );
    } else {
        base.setup_files = inherited_setup;
        base.global_setup = inherited_global;
    }
}

fn default_object_options(program: &Program<'_>, ctx: &mut Ctx<'_, '_>) -> Result<Option<Options>> {
    shared::default_export_object(program, &ctx.bindings)
        .map(|object| project_options(object, ctx))
        .transpose()
}

fn argument_options(expression: &Expression<'_>, ctx: &mut Ctx<'_, '_>) -> Result<Option<Options>> {
    match object_expressions::expression_object_options(expression, ctx)? {
        Some(options) => Ok(Some(options)),
        None => expression_object(expression, &ctx.bindings)
            .map(|object| project_options(object, ctx))
            .transpose(),
    }
}

fn is_merge_config(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name == "mergeConfig"
    ) || matches!(
        &call.callee,
        Expression::StaticMemberExpression(member) if member.property.name == "mergeConfig"
    )
}
