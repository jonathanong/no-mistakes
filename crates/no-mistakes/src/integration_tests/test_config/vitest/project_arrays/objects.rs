use super::{shared, Ctx, ExprMap, ImportBinding, Options};
use crate::integration_tests::test_config::deadlines;
use crate::integration_tests::test_config::vitest::Extends;
use crate::integration_tests::types::DeadlineUnknownReason;
use crate::integration_tests::types::VitestSetupField;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;
use std::collections::BTreeSet;

mod calls;
mod config_extends;
mod dynamic_triggers;
mod exports;
mod members;
mod merge;
mod object_expressions;
mod properties;
mod property_options;
mod setup_dependencies;
mod setup_imports;
mod static_members;

pub(super) use config_extends::merged::{
    default_options as raw_config_options, raw_failure_reason,
};
use config_extends::resolve_config_extends;
use merge::merge_options;
pub(super) use object_expressions::expression_object_options;
use object_expressions::{imported_options, spread_options};
use setup_dependencies::setup_dependencies;

pub(super) fn project_options(
    object: &ObjectExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Options> {
    parse_options(object, ctx)
}

pub(super) fn expression_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &ExprMap<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let mut seen = BTreeSet::new();
    shared::expression_config_object(expression, bindings, &mut seen)
}

fn parse_options(object: &ObjectExpression<'_>, ctx: &mut Ctx<'_, '_>) -> Result<Options> {
    let mut options = Options::default();
    // Vitest treats setup fields as test-scoped whenever a nested `test`
    // object exists, regardless of declaration order.
    let nested_test = object.properties.iter().any(|property| {
        matches!(property, ObjectPropertyKind::ObjectProperty(property)
            if shared::property_key_name(&property.key).as_deref() == Some("test"))
    });
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                property_options::apply_property(&mut options, property, nested_test, ctx)?;
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
                        true,
                    );
                    options.extends = Some(Extends::Unknown {
                        path: ctx.path.to_path_buf(),
                        span: Some((spread.span.start, spread.span.end)),
                    });
                    if !ctx.is_test_object {
                        options.deadline_extends = options.extends.clone();
                    }
                }
            }
        }
    }
    if let Some(Extends::Unknown { path, span }) = &options.deadline_extends {
        let declaration = deadlines::unknown(
            DeadlineUnknownReason::UnresolvedInheritance,
            path,
            span.map(|(start, end)| oxc_span::Span::new(start, end)),
        );
        if options.deadlines.case.is_none() {
            options.deadlines.case = Some(declaration.clone());
        }
        if options.deadlines.hook.is_none() {
            options.deadlines.hook = Some(declaration);
        }
    }
    resolve_config_extends(&mut options, ctx)?;
    Ok(options)
}
