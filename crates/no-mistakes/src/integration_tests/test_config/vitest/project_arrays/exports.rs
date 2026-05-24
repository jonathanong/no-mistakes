use super::{
    array_options, body_return_options, expression_statement_options, helper_expression_options,
    import_bindings, top_level_function_bodies, Ctx,
};
use crate::integration_tests::test_config::vitest::shared;
use crate::integration_tests::test_config::vitest::Options;
use oxc_ast::ast::{BindingPattern, Declaration, ExportDefaultDeclarationKind, Program, Statement};
use std::path::Path;

pub(super) fn exported_options(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    exported: &str,
) -> Vec<Options> {
    let mut ctx = Ctx {
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: parent.resolver,
        path,
        root: parent.root,
        seen: parent.seen,
    };
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) if exported != "default" => {
                if let Some(options) = named_export_options(export, exported, &mut ctx) {
                    return options;
                }
            }
            Statement::ExportDefaultDeclaration(export) if exported == "default" => {
                return default_export_options(&export.declaration, &mut ctx);
            }
            _ => {}
        }
    }
    Vec::new()
}

fn named_export_options(
    export: &oxc_ast::ast::ExportNamedDeclaration<'_>,
    exported: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<Options>> {
    if let Some(declaration) = &export.declaration {
        return declaration_options(declaration, exported, ctx);
    }
    for specifier in &export.specifiers {
        if specifier.exported.name() != exported {
            continue;
        }
        let local = specifier.local.name().to_string();
        return ctx
            .bindings
            .get(&local)
            .copied()
            .map(|expression| helper_expression_options(expression, ctx))
            .or_else(|| {
                ctx.functions
                    .get(&local)
                    .copied()
                    .map(|body| body_return_options(body, ctx))
            });
    }
    None
}

fn declaration_options(
    declaration: &Declaration<'_>,
    exported: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<Options>> {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            declaration.declarations.iter().find_map(|declarator| {
                if binding_name(&declarator.id) == Some(exported) {
                    declarator
                        .init
                        .as_ref()
                        .map(|init| helper_expression_options(init, ctx))
                } else {
                    None
                }
            })
        }
        Declaration::FunctionDeclaration(function)
            if function.id.as_ref().map(|id| id.name.as_str()) == Some(exported) =>
        {
            Some(
                function
                    .body
                    .as_ref()
                    .map(|body| body_return_options(body, ctx))
                    .unwrap_or_default(),
            )
        }
        _ => None,
    }
}

fn default_export_options(
    export: &ExportDefaultDeclarationKind<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<Options> {
    match export {
        ExportDefaultDeclarationKind::Identifier(identifier) => ctx
            .bindings
            .get(identifier.name.as_str())
            .copied()
            .map(|expression| helper_expression_options(expression, ctx))
            .or_else(|| {
                ctx.functions
                    .get(identifier.name.as_str())
                    .copied()
                    .map(|body| body_return_options(body, ctx))
            })
            .unwrap_or_default(),
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) if arrow.expression => {
            expression_statement_options(&arrow.body, ctx)
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            body_return_options(&arrow.body, ctx)
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => function
            .body
            .as_ref()
            .map(|body| body_return_options(body, ctx))
            .unwrap_or_default(),
        ExportDefaultDeclarationKind::ArrayExpression(array) => array_options(array, ctx),
        _ => Vec::new(),
    }
}

fn binding_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
