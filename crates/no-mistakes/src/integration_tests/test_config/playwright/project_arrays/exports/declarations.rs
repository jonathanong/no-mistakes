use super::{body_return_options, helper_expression_options, Ctx, Options};
use crate::integration_tests::test_config::shared;
use anyhow::Result;
use oxc_ast::ast::{BindingPattern, Declaration, Expression, FunctionBody, VariableDeclarator};

pub(super) fn named_export_options(
    export: &oxc_ast::ast::ExportNamedDeclaration<'_>,
    exported: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Result<Vec<Options>>> {
    if let Some(declaration) = &export.declaration {
        return declaration_options(declaration, exported, ctx);
    }
    for specifier in &export.specifiers {
        if specifier.export_kind.is_type() {
            continue;
        }
        if specifier.exported.name() != exported {
            continue;
        }
        if let Some(source) = &export.source {
            return Some(super::imported_options(
                &super::super::ImportBinding {
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
            return Some(super::imported_options(&import, ctx));
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
                if let Some(options) = declarator_options(declarator, exported, ctx) {
                    return Some(options);
                }
            }
            None
        }
        Declaration::FunctionDeclaration(function)
            if function.id.as_ref().map(|id| id.name.as_str()) == Some(exported)
                && function.body.is_some() =>
        {
            Some(default_function_options(function.body.as_deref(), ctx))
        }
        _ => None,
    }
}

pub(super) fn default_function_options(
    body: Option<&FunctionBody<'_>>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    match body {
        Some(body) => body_return_options(body, ctx),
        None => Ok(Vec::new()),
    }
}

fn declarator_options(
    declarator: &VariableDeclarator<'_>,
    exported: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Result<Vec<Options>>> {
    if binding_name(&declarator.id) == Some(exported) {
        return declarator
            .init
            .as_ref()
            .map(|init| helper_expression_options(init, ctx));
    }
    destructured_expression(&declarator.id, declarator.init.as_ref()?, exported, ctx)
        .map(|expression| helper_expression_options(expression, ctx))
}

fn destructured_expression<'a>(
    binding: &BindingPattern<'a>,
    init: &'a Expression<'a>,
    exported: &str,
    ctx: &Ctx<'a, '_>,
) -> Option<&'a Expression<'a>> {
    let BindingPattern::ObjectPattern(pattern) = binding else {
        return None;
    };
    let object = super::super::objects::expression_object(init, &ctx.bindings)?;
    for binding_property in &pattern.properties {
        if binding_property.computed {
            continue;
        }
        let key = shared::property_key_name(&binding_property.key)?;
        let BindingPattern::BindingIdentifier(identifier) = &binding_property.value else {
            continue;
        };
        if identifier.name != exported {
            continue;
        }
        let Some(value) = shared::property_expression_deep(object, key.as_str(), &ctx.bindings)
        else {
            continue;
        };
        return Some(value);
    }
    None
}

fn binding_name<'a>(binding: &'a BindingPattern<'a>) -> Option<&'a str> {
    match binding {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}
