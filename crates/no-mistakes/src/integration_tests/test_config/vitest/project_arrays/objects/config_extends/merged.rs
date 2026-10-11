use super::super::{project_options, Ctx, Options};
use anyhow::Result;
use oxc_ast::ast::{Expression, Program, Statement};
use oxc_span::GetSpan;

mod public_calls;
use public_calls::public_config_call;

/// Parse only static `mergeConfig` arguments. Unknown arguments make the
/// inherited config conservative rather than accepting a partial merge.
pub(in crate::integration_tests::test_config::vitest::project_arrays) fn default_options(
    program: &Program<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Some(Statement::ExportDefaultDeclaration(export)) = program
        .body
        .iter()
        .find(|statement| matches!(statement, Statement::ExportDefaultDeclaration(_)))
    else {
        // Retain the existing CommonJS structural interpreter, without SDK deadline proof.
        let Some(object) = super::super::shared::default_export_object(program, &ctx.bindings)
        else {
            return Ok(None);
        };
        let mut options = project_options(object, ctx)?;
        crate::integration_tests::test_config::deadlines::obscure(
            &mut options.deadlines,
            ctx.path,
            object.span,
            crate::integration_tests::types::DeadlineUnknownReason::UnsupportedConfigCall,
            true,
        );
        return Ok(Some(options));
    };
    let Some(expression) = export.declaration.as_expression() else {
        return Ok(None);
    };
    argument_options(expression, program, ctx)
}

fn merge_options(base: &mut Options, next: Options) {
    let mut deadlines = base.deadlines.clone();
    deadlines.overlay(next.deadlines.clone());
    let inherited_setup = base.setup_files.take();
    let inherited_global = base.global_setup.take();
    let next_setup = next.setup_files.clone();
    let next_global = next.global_setup.clone();
    let next_setup_cleared = next.setup_files_cleared;
    let next_global_cleared = next.global_setup_cleared;
    let nested = next.nested_test_scope;
    super::super::merge::merge_options(base, next);
    base.deadlines = deadlines;
    if nested {
        base.setup_files = (!next_setup_cleared)
            .then(|| {
                crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
                    inherited_setup,
                    next_setup,
                )
            })
            .flatten();
        base.global_setup = (!next_global_cleared)
            .then(|| {
                crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
                    inherited_global,
                    next_global,
                )
            })
            .flatten();
    } else {
        base.setup_files = inherited_setup;
        base.global_setup = inherited_global;
    }
}

fn argument_options(
    expression: &Expression<'_>,
    program: &Program<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ObjectExpression(object) => project_options(object, ctx).map(Some),
        // The existing ExprMap contains let/var and object aliases without write/escape proof.
        // Preserve ordinary structural options; the existing interpreter marks deadlines unknown.
        Expression::Identifier(_) => super::super::expression_object_options(expression, ctx),
        Expression::CallExpression(call) => {
            let role = public_config_call(&call.callee, program, ctx);
            match role.as_deref() {
                Some("defineConfig" | "defineProject") if call.arguments.len() == 1 => {
                    match call.arguments[0].as_expression() {
                        Some(value) => argument_options(value, program, ctx),
                        None => Ok(None),
                    }
                }
                _ if role.as_deref() == Some("mergeConfig")
                    || structural_merge_call(&call.callee) =>
                {
                    // Structural discovery keeps its existing call shape contract. Only the
                    // public two-config call (plus literal isRoot) can prove deadlines.
                    let proved = role.as_deref() == Some("mergeConfig")
                        && (call.arguments.len() == 2
                            || (call.arguments.len() == 3
                                && call.arguments[2].as_expression().is_some_and(|value| {
                                    matches!(
                                        crate::codebase::ts_source::unwrap_ts_wrappers(value),
                                        Expression::BooleanLiteral(_)
                                    )
                                })));
                    let count = if proved { 2 } else { call.arguments.len() };
                    let mut merged = Options::default();
                    for argument in call.arguments.iter().take(count) {
                        let Some(value) = argument.as_expression() else {
                            return Ok(None);
                        };
                        let Some(options) = argument_options(value, program, ctx)? else {
                            return Ok(None);
                        };
                        merge_options(&mut merged, options);
                    }
                    if !proved {
                        obscure_call_deadlines(&mut merged, expression, ctx);
                    }
                    Ok((!call.arguments.is_empty()).then_some(merged))
                }
                _ => {
                    let mut options = super::super::expression_object(expression, &ctx.bindings)
                        .map(|object| project_options(object, ctx))
                        .transpose()?;
                    if let Some(options) = &mut options {
                        obscure_call_deadlines(options, expression, ctx);
                    }
                    Ok(options)
                }
            }
        }
        _ => Ok(None),
    }
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn raw_failure_reason(
    program: &Program<'_>,
) -> crate::integration_tests::types::DeadlineUnknownReason {
    use crate::integration_tests::types::DeadlineUnknownReason;
    for statement in &program.body {
        if let Statement::ExportDefaultDeclaration(export) = statement {
            if export.declaration.as_expression().is_some_and(|value| {
                matches!(
                    crate::codebase::ts_source::unwrap_ts_wrappers(value),
                    Expression::Identifier(_)
                )
            }) {
                return DeadlineUnknownReason::UnprovedBinding;
            }
        }
    }
    DeadlineUnknownReason::UnsupportedConfigCall
}

fn structural_merge_call(callee: &Expression<'_>) -> bool {
    matches!(callee, Expression::Identifier(identifier) if identifier.name == "mergeConfig")
        || matches!(callee, Expression::StaticMemberExpression(member) if member.property.name == "mergeConfig")
}

fn obscure_call_deadlines(options: &mut Options, expression: &Expression<'_>, ctx: &Ctx<'_, '_>) {
    crate::integration_tests::test_config::deadlines::obscure(
        &mut options.deadlines,
        ctx.path,
        expression.span(),
        crate::integration_tests::types::DeadlineUnknownReason::UnsupportedConfigCall,
        true,
    );
}
