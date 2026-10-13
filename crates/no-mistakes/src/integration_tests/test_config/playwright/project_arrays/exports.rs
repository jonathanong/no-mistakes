use super::{
    array_options, body_return_options, expression_options, helper_expression_options,
    import_bindings, imported_options, Ctx,
};
use crate::integration_tests::test_config::playwright::Options;
use anyhow::Result;
use oxc_ast::ast::{ArrowFunctionBody, ExportDefaultDeclarationKind, Program, Statement};
use std::path::Path;

mod commonjs;
use crate::integration_tests::test_config::shared_project_arrays::declarations::{
    self, DeclarationOps,
};
use crate::integration_tests::test_config::shared_project_arrays::export_scan::{
    self, ExportScanOps,
};
use commonjs::commonjs_default_expression;

fn declaration_ops() -> DeclarationOps<Options> {
    DeclarationOps {
        expression_options: helper_expression_options,
        body_options: body_return_options,
        imported_options,
        expression_object: super::objects::expression_object,
    }
}

pub(super) fn exported_options(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    exported: &str,
) -> Result<Vec<Options>> {
    Ok(exported_options_lookup(program, source, path, parent, exported)?.unwrap_or_default())
}

fn export_scan_ops() -> ExportScanOps<Options> {
    ExportScanOps {
        declarations: declaration_ops(),
        import_bindings,
        default_options: default_export_options,
        commonjs_options,
        preserve_test_scope: false,
    }
}

fn exported_options_lookup(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    exported: &str,
) -> Result<Option<Vec<Options>>> {
    export_scan::exported_options_lookup(
        program,
        source,
        path,
        parent,
        exported,
        &export_scan_ops(),
        exported_options_lookup,
    )
}

fn commonjs_options(
    _program: &Program<'_>,
    statement: &Statement<'_>,
    exported: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Result<Vec<Options>>> {
    if exported != "default" {
        return None;
    }
    let Statement::ExpressionStatement(statement) = statement else {
        return None;
    };
    commonjs_default_expression(&statement.expression)
        .map(|expression| expression_options(expression, ctx))
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
            _ => expression_options(arrow.body.to_expression(), ctx),
        },
        ExportDefaultDeclarationKind::CallExpression(call) if call.arguments.is_empty() => {
            super::calls::call_options(&call.callee, ctx)
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            declarations::default_function_options(
                function.body.as_deref(),
                ctx,
                &declaration_ops(),
            )
        }
        ExportDefaultDeclarationKind::ArrayExpression(array) => array_options(array, ctx),
        ExportDefaultDeclarationKind::ObjectExpression(object) => {
            Ok(vec![super::objects::project_object_options(object, ctx)?])
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            expression_options(&parenthesized.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            expression_options(&expression.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            expression_options(&expression.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSTypeAssertion(expression) => {
            expression_options(&expression.expression, ctx)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(expression) => {
            expression_options(&expression.expression, ctx)
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
