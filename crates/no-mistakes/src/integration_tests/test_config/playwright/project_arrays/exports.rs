use super::{
    array_options, body_return_options, expression_options, expression_statement_options,
    helper_expression_options, import_bindings, imported_options, top_level_function_bodies, Ctx,
};
use crate::ast;
use crate::integration_tests::test_config::playwright::Options;
use crate::integration_tests::test_config::shared;
use anyhow::Result;
use oxc_ast::ast::{
    AssignmentTarget, ExportDefaultDeclarationKind, Expression, Program, Statement,
};
use std::collections::BTreeSet;
use std::path::Path;

mod declarations;

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
                if let Some(options) =
                    declarations::named_export_options(export, exported, &mut ctx)
                {
                    return options.map(Some);
                }
            }
            Statement::ExportAllDeclaration(export)
                if !export.export_kind.is_type()
                    && exported != "default"
                    && export.exported.is_none() =>
            {
                export_all_sources.push(export.source.value.to_string());
            }
            Statement::ExportDefaultDeclaration(export) if exported == "default" => {
                return default_export_options(&export.declaration, &mut ctx).map(Some);
            }
            Statement::ExpressionStatement(statement) if exported == "default" => {
                if let Some(expression) = commonjs_default_expression(&statement.expression) {
                    return expression_options(expression, &mut ctx).map(Some);
                }
            }
            _ => {}
        }
    }
    let mut resolved = None;
    for source in export_all_sources {
        let binding = super::ImportBinding {
            source,
            imported: exported.to_string(),
        };
        let Some(options) = imported_options_lookup(&binding, path, &mut ctx)? else {
            continue;
        };
        if resolved.is_some() {
            return Ok(None);
        }
        resolved = Some(options);
    }
    Ok(resolved)
}

fn imported_options_lookup(
    import: &super::ImportBinding,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Vec<Options>>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(None);
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(None);
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(None),
        Ok(source) => crate::integration_tests::runner_config::with_program(
            &path,
            &source,
            |program, source| {
                exported_options_lookup(program, source, &path, ctx, &import.imported)
            },
        )
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn commonjs_default_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    let Expression::AssignmentExpression(assignment) = expression else {
        return None;
    };
    if assignment_target_path(&assignment.left)
        .as_deref()
        .is_none_or(|parts| parts != ["module", "exports"])
    {
        return None;
    }
    Some(&assignment.right)
}

fn assignment_target_path(target: &AssignmentTarget<'_>) -> Option<Vec<String>> {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => {
            let mut parts = ast::expression_path(&member.object)?;
            parts.push(member.property.name.to_string());
            Some(parts)
        }
        _ => None,
    }
}

fn default_export_options(
    export: &ExportDefaultDeclarationKind<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    match export {
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            default_identifier_options(identifier.name.as_str(), ctx)
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) if arrow.expression => {
            expression_statement_options(&arrow.body, ctx)
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            body_return_options(&arrow.body, ctx)
        }
        ExportDefaultDeclarationKind::CallExpression(call) if call.arguments.is_empty() => {
            super::calls::call_options(&call.callee, ctx)
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            declarations::default_function_options(function.body.as_deref(), ctx)
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
