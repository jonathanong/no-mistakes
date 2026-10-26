use super::{shared, Ctx, ExprMap, Options};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::test_config::deadlines;
use crate::integration_tests::types::DeadlineUnknownReason;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;
use std::collections::BTreeSet;

use crate::integration_tests::test_config::shared_project_arrays::object_calls::{
    self as calls, ObjectCallOps,
};
mod exports;
mod properties;
use self::project_object_options as project_options;
use crate::integration_tests::test_config::shared_project_arrays::object_members::{
    self as members, ObjectMemberOps,
};
use exports::imported_object_options;
use exports::imported_object_options as imported_options;
use properties::merge_property;

fn object_call_ops() -> ObjectCallOps<Options> {
    ObjectCallOps {
        expression_object_options,
        import_bindings: super::import_bindings,
    }
}

fn object_member_ops() -> ObjectMemberOps<Options> {
    ObjectMemberOps {
        expression_object,
        project_options,
        imported_options,
        import_bindings: super::import_bindings,
        default_identifier_reexport: false,
    }
}

pub(super) fn project_object_options(
    object: &ObjectExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Options> {
    let mut options = Options::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                let name = shared::property_key_name(&property.key);
                if property.computed && name.is_none() {
                    deadlines::obscure(
                        &mut options.deadlines,
                        ctx.path,
                        property.span(),
                        DeadlineUnknownReason::ComputedProperty,
                        false,
                    );
                    continue;
                }
                if property.method || property.kind != oxc_ast::ast::PropertyKind::Init {
                    if name.as_deref() == Some("timeout") {
                        options.deadlines.case = Some(deadlines::unknown(
                            DeadlineUnknownReason::Accessor,
                            ctx.path,
                            Some(property.span()),
                        ));
                    }
                    continue;
                }
                merge_property(
                    &mut options,
                    name.as_deref(),
                    &property.value,
                    property.span().start,
                    ctx,
                )?;
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(imported) = spread_options(&spread.argument, ctx)? {
                    merge_options(&mut options, imported);
                } else {
                    deadlines::obscure(
                        &mut options.deadlines,
                        ctx.path,
                        spread.span(),
                        DeadlineUnknownReason::OpaqueSpread,
                        false,
                    );
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
    let mut result = expression_object_options_inner(expression, ctx)?;
    if !matches!(
        unwrap_ts_wrappers(expression),
        Expression::ObjectExpression(_)
    ) {
        if let Some(options) = &mut result {
            deadlines::unproved_case(&mut options.deadlines, ctx.path, expression.span());
        }
    }
    Ok(result)
}

fn expression_object_options_inner(
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
            return members::namespace_member_options(member, ctx, &object_member_ops());
        }
        Expression::CallExpression(call) if call.arguments.is_empty() => {
            return calls::call_object_options(&call.callee, ctx, &object_call_ops());
        }
        Expression::CallExpression(_) => return Ok(None),
        _ => {}
    }
    let Some(object) = expression_object(expression, &ctx.bindings) else {
        return Ok(None);
    };
    project_object_options(object, ctx).map(Some)
}

pub(super) fn expression_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &ExprMap<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let mut seen = BTreeSet::new();
    shared::expression_config_object(expression, bindings, &mut seen)
}

fn merge_options(base: &mut Options, next: Options) {
    base.deadlines.overlay(next.deadlines);
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
