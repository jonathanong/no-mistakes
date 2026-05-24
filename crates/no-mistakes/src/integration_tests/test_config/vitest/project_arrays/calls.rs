use super::{body_return_options, helper_expression_options, imported_options, Ctx};
use crate::integration_tests::test_config::vitest::Options;
use oxc_ast::ast::Expression;

pub(super) fn call_options(callee: &Expression<'_>, ctx: &mut Ctx<'_, '_>) -> Vec<Options> {
    match callee {
        Expression::Identifier(identifier) => {
            call_identifier_options(identifier.name.as_str(), ctx)
        }
        Expression::StaticMemberExpression(member) => namespace_call_options(member, ctx),
        _ => Vec::new(),
    }
}

fn namespace_call_options(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Vec<Options> {
    let Expression::Identifier(object) = &member.object else {
        return Vec::new();
    };
    ctx.imports
        .get(object.name.as_str())
        .filter(|import| import.imported == "*")
        .cloned()
        .map(|import| {
            imported_options(
                &super::ImportBinding {
                    source: import.source,
                    imported: member.property.name.to_string(),
                },
                ctx,
            )
        })
        .unwrap_or_default()
}

fn call_identifier_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Vec<Options> {
    let key = format!("call:{name}");
    if !ctx.local_seen.insert(key.clone()) {
        return Vec::new();
    }
    let result = local_call_options(name, ctx);
    ctx.local_seen.remove(&key);
    result
}

fn local_call_options(name: &str, ctx: &mut Ctx<'_, '_>) -> Vec<Options> {
    if let Some(expression) = ctx.bindings.get(name).copied() {
        let options = helper_expression_options(expression, ctx);
        if !options.is_empty() {
            return options;
        }
    }
    if let Some(body) = ctx.functions.get(name).copied() {
        let options = body_return_options(body, ctx);
        if !options.is_empty() {
            return options;
        }
    }
    ctx.imports
        .get(name)
        .cloned()
        .map(|import| imported_options(&import, ctx))
        .unwrap_or_default()
}
