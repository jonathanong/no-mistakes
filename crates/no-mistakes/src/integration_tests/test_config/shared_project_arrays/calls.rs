use super::{Context, ImportBinding};
use anyhow::Result;
use oxc_ast::ast::{Expression, FunctionBody};

type ExpressionOptions<T> = fn(&Expression<'_>, &mut Context<'_, '_>) -> Result<Vec<T>>;
type BodyOptions<T> = fn(&FunctionBody<'_>, &mut Context<'_, '_>) -> Result<Vec<T>>;
type ImportedOptions<T> = fn(&ImportBinding, &mut Context<'_, '_>) -> Result<Vec<T>>;

pub(in crate::integration_tests::test_config) fn call_options<T>(
    callee: &Expression<'_>,
    ctx: &mut Context<'_, '_>,
    expression_options: ExpressionOptions<T>,
    body_options: BodyOptions<T>,
    imported_options: ImportedOptions<T>,
) -> Result<Vec<T>> {
    match callee {
        Expression::Identifier(identifier) => call_identifier_options(
            identifier.name.as_str(),
            ctx,
            expression_options,
            body_options,
            imported_options,
        ),
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(object) = &member.object else {
                return Ok(Vec::new());
            };
            let Some(import) = ctx.imports.get(object.name.as_str()).cloned() else {
                return Ok(Vec::new());
            };
            if import.imported != "*" {
                return Ok(Vec::new());
            }
            imported_options(
                &ImportBinding {
                    source: import.source,
                    imported: member.property.name.to_string(),
                },
                ctx,
            )
        }
        _ => Ok(Vec::new()),
    }
}

fn call_identifier_options<T>(
    name: &str,
    ctx: &mut Context<'_, '_>,
    expression_options: ExpressionOptions<T>,
    body_options: BodyOptions<T>,
    imported_options: ImportedOptions<T>,
) -> Result<Vec<T>> {
    let key = format!("call:{name}");
    if !ctx.local_seen.insert(key.clone()) {
        return Ok(Vec::new());
    }
    let result = local_call_options(
        name,
        ctx,
        expression_options,
        body_options,
        imported_options,
    );
    ctx.local_seen.remove(&key);
    result
}

fn local_call_options<T>(
    name: &str,
    ctx: &mut Context<'_, '_>,
    expression_options: ExpressionOptions<T>,
    body_options: BodyOptions<T>,
    imported_options: ImportedOptions<T>,
) -> Result<Vec<T>> {
    if let Some(expression) = ctx.bindings.get(name).copied() {
        let options = expression_options(expression, ctx)?;
        if !options.is_empty() {
            return Ok(options);
        }
    }
    if let Some(body) = ctx.functions.get(name).copied() {
        let options = body_options(body, ctx)?;
        if !options.is_empty() {
            return Ok(options);
        }
    }
    if let Some(import) = ctx.imports.get(name).cloned() {
        imported_options(&import, ctx)
    } else {
        Ok(Vec::new())
    }
}
