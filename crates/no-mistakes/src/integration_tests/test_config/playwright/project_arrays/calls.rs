use super::{body_return_options, helper_expression_options, imported_options, Ctx};
use crate::integration_tests::test_config::playwright::Options;
use anyhow::Result;
use oxc_ast::ast::Expression;

pub(super) fn call_options(callee: &Expression<'_>, ctx: &mut Ctx<'_, '_>) -> Result<Vec<Options>> {
    match callee {
        Expression::Identifier(identifier) => {
            call_identifier_options(identifier.name.as_str(), ctx)
        }
        Expression::StaticMemberExpression(member) => namespace_call_options(member, ctx),
        _ => Ok(Vec::new()),
    }
}

fn namespace_call_options(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
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
        &super::ImportBinding {
            source: import.source,
            imported: member.property.name.to_string(),
        },
        ctx,
    )
}

fn call_identifier_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Result<Vec<Options>> {
    let key = format!("call:{name}");
    if !ctx.local_seen.insert(key.clone()) {
        return Ok(Vec::new());
    }
    let result = local_call_options(name, ctx);
    ctx.local_seen.remove(&key);
    result
}

fn local_call_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Result<Vec<Options>> {
    if let Some(expression) = ctx.bindings.get(name).copied() {
        let options = helper_expression_options(expression, ctx)?;
        if !options.is_empty() {
            return Ok(options);
        }
    }
    if let Some(body) = ctx.functions.get(name).copied() {
        let options = body_return_options(body, ctx)?;
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
