use super::{shared, Ctx, ExprMap, Options};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use std::collections::BTreeSet;

mod calls;
mod exports;
mod members;
use exports::imported_object_options;

pub(super) fn project_object_options(
    object: &ObjectExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Options> {
    let mut options = Options::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                let name = (!property.computed && !property.method)
                    .then(|| shared::property_key_name(&property.key))
                    .flatten();
                merge_property(&mut options, name.as_deref(), &property.value, ctx)?;
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(imported) = spread_options(&spread.argument, ctx)? {
                    merge_options(&mut options, imported);
                }
            }
        }
    }
    Ok(options)
}

fn spread_options(expression: &Expression<'_>, ctx: &mut Ctx<'_, '_>) -> Result<Option<Options>> {
    expression_object_options(expression, ctx)
}

pub(super) fn expression_object_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if let Some(import) = ctx.imports.get(name).cloned() {
                return imported_object_options(&import, ctx);
            }
            if !ctx.object_seen.insert(name.to_string()) {
                return Ok(None);
            }
            let result = match ctx
                .bindings
                .get(name)
                .and_then(|expression| expression_object(expression, &ctx.bindings))
            {
                Some(object) => project_object_options(object, ctx).map(Some),
                None => Ok(None),
            };
            ctx.object_seen.remove(name);
            return result;
        }
        Expression::StaticMemberExpression(member) => {
            return members::namespace_member_options(member, ctx);
        }
        Expression::CallExpression(call) if call.arguments.is_empty() => {
            return calls::call_object_options(&call.callee, ctx);
        }
        Expression::CallExpression(_) => return Ok(None),
        _ => {}
    }
    let Some(object) = expression_object(expression, &ctx.bindings) else {
        return Ok(None);
    };
    project_object_options(object, ctx).map(Some)
}

fn merge_property(
    options: &mut Options,
    name: Option<&str>,
    value: &Expression<'_>,
    ctx: &Ctx<'_, '_>,
) -> Result<()> {
    let value = shared::expression_value(value, &ctx.bindings);
    match name {
        Some("name") => options.name = shared::optional_string(value, ctx.source),
        Some("testDir") => {
            options.test_dir = Some(shared::required_string(value, ctx.source, "testDir")?);
        }
        Some("testMatch") => {
            let test_match = shared::inferred_string_or_array(value, ctx.source, "testMatch")?;
            if test_match.is_empty() {
                anyhow::bail!("expected string literal or string array for testMatch");
            }
            options.test_match = Some(test_match);
        }
        Some("testIgnore") => {
            options.test_ignore = Some(shared::inferred_string_or_array(
                value,
                ctx.source,
                "testIgnore",
            )?);
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn expression_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &ExprMap<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let mut seen = BTreeSet::new();
    shared::expression_config_object(expression, bindings, &mut seen)
}

fn merge_options(base: &mut Options, next: Options) {
    if next.name.is_some() {
        base.name = next.name;
    }
    if next.test_dir.is_some() {
        base.test_dir = next.test_dir;
    }
    if next.test_match.is_some() {
        base.test_match = next.test_match;
    }
    if next.test_ignore.is_some() {
        base.test_ignore = next.test_ignore;
    }
}
