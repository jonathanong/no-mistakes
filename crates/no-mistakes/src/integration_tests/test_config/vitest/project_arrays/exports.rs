use super::{
    array_options, body_return_options, helper_expression_options, import_bindings,
    imported_options, top_level_function_bodies, Ctx,
};
use crate::integration_tests::test_config::vitest::shared;
use crate::integration_tests::test_config::vitest::Options;
use anyhow::Result;
use oxc_ast::ast::{ArrowFunctionBody, ExportDefaultDeclarationKind, Program, Statement};
use std::collections::BTreeSet;
use std::path::Path;

pub(in crate::integration_tests::test_config::vitest::project_arrays) mod commonjs;
#[path = "../../shared_project_arrays/declarations.rs"]
mod declarations;
mod workspace;
use commonjs::commonjs_exported_expression;
use declarations::{
    declaration_options, default_function_options, export_from_options, named_export_options,
};
pub(super) use workspace::workspace_default_options;

pub(super) fn exported_options(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    exported: &str,
) -> Result<Vec<Options>> {
    Ok(exported_options_lookup(program, source, path, parent, exported)?.unwrap_or_default())
}

fn exported_options_lookup(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    exported: &str,
) -> Result<Option<Vec<Options>>> {
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        is_test_object: parent.is_test_object,
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: parent.resolver,
        path,
        seen: parent.seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    let mut export_all_sources = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if export.export_kind.is_type() {
                    continue;
                }
                if let Some(options) = named_export_options(export, exported, &mut ctx) {
                    return options.map(Some);
                }
            }
            Statement::ExportDeclaration(export) => {
                if let Some(options) = declaration_options(&export.declaration, exported, &mut ctx)
                {
                    return options.map(Some);
                }
            }
            Statement::ExportFromDeclaration(export) => {
                if let Some(options) = export_from_options(export, exported, &mut ctx) {
                    return options.map(Some);
                }
            }
            Statement::ExportAllDeclaration(export)
                if exported != "default"
                    && export.exported.is_none()
                    && !export.export_kind.is_type() =>
            {
                export_all_sources.push(export.source.value.to_string());
            }
            Statement::ExportDefaultDeclaration(export) if exported == "default" => {
                return default_export_options(&export.declaration, &mut ctx).map(Some);
            }
            Statement::ExpressionStatement(_) => {
                if let Some(expression) =
                    commonjs_exported_expression(program, exported, &ctx.bindings)
                {
                    return super::expression_options(expression, &mut ctx).map(Some);
                }
            }
            _ => {}
        }
    }
    crate::integration_tests::test_config::shared_project_arrays::resolve_imported_star_exports(
        export_all_sources,
        exported,
        path,
        &mut ctx,
        exported_options_lookup,
    )
}

fn default_export_options(
    export: &ExportDefaultDeclarationKind<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    match export {
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            default_identifier_options(identifier.name.as_str(), ctx)
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => match &arrow.body {
            ArrowFunctionBody::FunctionBody(body) => body_return_options(body, ctx),
            _ => super::expression_options(arrow.body.to_expression(), ctx),
        },
        ExportDefaultDeclarationKind::CallExpression(call) if call.arguments.is_empty() => {
            super::calls::call_options(&call.callee, ctx)
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            default_function_options(function.body.as_deref(), ctx)
        }
        ExportDefaultDeclarationKind::ArrayExpression(array) => array_options(array, ctx),
        ExportDefaultDeclarationKind::ObjectExpression(object) => {
            Ok(vec![super::objects::project_options(object, ctx)?])
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            super::expression_options(&parenthesized.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            super::expression_options(&expression.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            super::expression_options(&expression.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSTypeAssertion(expression) => {
            super::expression_options(&expression.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(expression) => {
            super::expression_options(&expression.expression, ctx)
        }
        _ => Ok(Vec::new()),
    }
}

fn default_identifier_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Result<Vec<Options>> {
    if let Some(expression) = ctx.bindings.get(name).copied() {
        helper_expression_options(expression, ctx)
    } else if let Some(body) = ctx.functions.get(name).copied() {
        body_return_options(body, ctx)
    } else if let Some(import) = ctx.imports.get(name).cloned() {
        imported_options(&import, ctx)
    } else {
        Ok(Vec::new())
    }
}
