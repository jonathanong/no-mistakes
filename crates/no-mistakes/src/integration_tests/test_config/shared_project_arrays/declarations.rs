use super::{Context, ExprMap, ImportBinding};
use crate::integration_tests::test_config::shared;
use anyhow::Result;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, FunctionBody, ObjectExpression, VariableDeclarator,
};

type ExpressionOptions<T> = fn(&Expression<'_>, &mut Context<'_, '_>) -> Result<Vec<T>>;
type BodyOptions<T> = fn(&FunctionBody<'_>, &mut Context<'_, '_>) -> Result<Vec<T>>;
type ImportedOptions<T> = fn(&ImportBinding, &mut Context<'_, '_>) -> Result<Vec<T>>;
type ExpressionObject =
    for<'a> fn(&'a Expression<'a>, &ExprMap<'a>) -> Option<&'a ObjectExpression<'a>>;

pub(in crate::integration_tests::test_config) struct DeclarationOps<T> {
    pub expression_options: ExpressionOptions<T>,
    pub body_options: BodyOptions<T>,
    pub imported_options: ImportedOptions<T>,
    pub expression_object: ExpressionObject,
}

pub(in crate::integration_tests::test_config) fn named_export_options<T>(
    export: &oxc_ast::ast::ExportNamedDeclaration<'_>,
    exported: &str,
    ctx: &mut Context<'_, '_>,
    ops: &DeclarationOps<T>,
) -> Option<Result<Vec<T>>> {
    for specifier in &export.specifiers {
        if specifier.export_kind.is_type() || specifier.exported.name() != exported {
            continue;
        }
        let local = specifier.local.name().to_string();
        if let Some(expression) = ctx.bindings.get(&local).copied() {
            return Some((ops.expression_options)(expression, ctx));
        }
        if let Some(body) = ctx.functions.get(&local).copied() {
            return Some((ops.body_options)(body, ctx));
        }
        if let Some(import) = ctx.imports.get(&local).cloned() {
            return Some((ops.imported_options)(&import, ctx));
        }
        return None;
    }
    None
}

pub(in crate::integration_tests::test_config) fn declaration_options<T>(
    declaration: &Declaration<'_>,
    exported: &str,
    ctx: &mut Context<'_, '_>,
    ops: &DeclarationOps<T>,
) -> Option<Result<Vec<T>>> {
    match declaration {
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                if let Some(options) = declarator_options(declarator, exported, ctx, ops) {
                    return Some(options);
                }
            }
            None
        }
        Declaration::FunctionDeclaration(function)
            if function.id.as_ref().map(|id| id.name.as_str()) == Some(exported)
                && function.body.is_some() =>
        {
            Some(default_function_options(function.body.as_deref(), ctx, ops))
        }
        _ => None,
    }
}

pub(in crate::integration_tests::test_config) fn export_from_options<T>(
    export: &oxc_ast::ast::ExportFromDeclaration<'_>,
    exported: &str,
    ctx: &mut Context<'_, '_>,
    ops: &DeclarationOps<T>,
) -> Option<Result<Vec<T>>> {
    if export.export_kind.is_type() {
        return None;
    }
    export.specifiers.iter().find_map(|specifier| {
        (!specifier.export_kind.is_type() && specifier.exported.name() == exported).then(|| {
            (ops.imported_options)(
                &ImportBinding {
                    source: export.source.value.to_string(),
                    imported: specifier.local.name().to_string(),
                },
                ctx,
            )
        })
    })
}

pub(in crate::integration_tests::test_config) fn default_function_options<T>(
    body: Option<&FunctionBody<'_>>,
    ctx: &mut Context<'_, '_>,
    ops: &DeclarationOps<T>,
) -> Result<Vec<T>> {
    match body {
        Some(body) => (ops.body_options)(body, ctx),
        None => Ok(Vec::new()),
    }
}

fn declarator_options<T>(
    declarator: &VariableDeclarator<'_>,
    exported: &str,
    ctx: &mut Context<'_, '_>,
    ops: &DeclarationOps<T>,
) -> Option<Result<Vec<T>>> {
    if binding_name(&declarator.id) == Some(exported) {
        return declarator
            .init
            .as_ref()
            .map(|init| (ops.expression_options)(init, ctx));
    }
    destructured_expression(
        &declarator.id,
        declarator.init.as_ref()?,
        exported,
        ctx,
        ops,
    )
    .map(|expression| (ops.expression_options)(expression, ctx))
}

fn destructured_expression<'a, T>(
    binding: &BindingPattern<'a>,
    init: &'a Expression<'a>,
    exported: &str,
    ctx: &Context<'a, '_>,
    ops: &DeclarationOps<T>,
) -> Option<&'a Expression<'a>> {
    let BindingPattern::ObjectPattern(pattern) = binding else {
        return None;
    };
    let object = (ops.expression_object)(init, &ctx.bindings)?;
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
