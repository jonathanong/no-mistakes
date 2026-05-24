use super::{
    array_options, body_return_options, expression_statement_options, helper_expression_options,
    import_bindings, imported_options, top_level_function_bodies, Ctx,
};
use crate::integration_tests::test_config::vitest::shared;
use crate::integration_tests::test_config::vitest::Options;
use anyhow::Result;
use oxc_ast::ast::{BindingPattern, Declaration, ExportDefaultDeclarationKind, Program, Statement};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn exported_options(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    exported: &str,
) -> Result<Vec<Options>> {
    let mut local_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings: shared::top_level_object_bindings(program),
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: parent.resolver,
        path,
        root: parent.root,
        seen: parent.seen,
        local_seen: &mut local_seen,
    };
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
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
    Ok(Vec::new())
}

fn named_export_options(
    export: &oxc_ast::ast::ExportNamedDeclaration<'_>,
    exported: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Result<Vec<Options>>> {
    if let Some(declaration) = &export.declaration {
        return declaration_options(declaration, exported, ctx);
    }
    for specifier in &export.specifiers {
        if specifier.exported.name() != exported {
            continue;
        }
        if let Some(source) = &export.source {
            return Some(imported_options(
                &super::ImportBinding {
                    source: source.value.to_string(),
                    imported: specifier.local.name().to_string(),
                },
                ctx,
            ));
        }
        let local = specifier.local.name().to_string();
        if let Some(expression) = ctx.bindings.get(&local).copied() {
            return Some(helper_expression_options(expression, ctx));
        }
        if let Some(body) = ctx.functions.get(&local).copied() {
            return Some(body_return_options(body, ctx));
        }
        if let Some(import) = ctx.imports.get(&local).cloned() {
            return Some(imported_options(&import, ctx));
        }
        return None;
    }
    None
}

fn declaration_options(
    declaration: &Declaration<'_>,
    exported: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Result<Vec<Options>>> {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                if binding_name(&declarator.id) == Some(exported) {
                    return declarator
                        .init
                        .as_ref()
                        .map(|init| helper_expression_options(init, ctx));
                }
            }
            None
        }
        Declaration::FunctionDeclaration(function)
            if function.id.as_ref().map(|id| id.name.as_str()) == Some(exported) =>
        {
            if let Some(body) = &function.body {
                Some(body_return_options(body, ctx))
            } else {
                Some(Ok(Vec::new()))
            }
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
            let name = identifier.name.as_str();
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
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) if arrow.expression => {
            expression_statement_options(&arrow.body, ctx)
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            body_return_options(&arrow.body, ctx)
        }
        ExportDefaultDeclarationKind::CallExpression(call) => {
            super::calls::call_options(&call.callee, ctx)
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            if let Some(body) = &function.body {
                body_return_options(body, ctx)
            } else {
                Ok(Vec::new())
            }
        }
        ExportDefaultDeclarationKind::ArrayExpression(array) => array_options(array, ctx),
        _ => Ok(Vec::new()),
    }
}

fn binding_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
