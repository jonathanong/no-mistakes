use super::{shared, Ctx, ExprMap, ImportBinding, Options};
use crate::integration_tests::types::VitestSetupField;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use std::collections::BTreeSet;

mod calls;
mod exports;
mod members;
mod merge;
mod object_expressions;
mod setup_dependencies;

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
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed || property.method {
                    continue;
                }
                let name = shared::property_key_name(&property.key);
                if name.as_deref() == Some("test") {
                    if let Some(test_options) = expression_object_options(&property.value, ctx)? {
                        options.name = None;
                        options.include = None;
                        options.exclude = None;
                        options.setup_files = None;
                        options.global_setup = None;
                        merge_options(&mut options, test_options);
                    }
                    continue;
                }
                merge_property(&mut options, name, &property.value, ctx)?;
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

fn merge_property(
    options: &mut Options,
    name: Option<String>,
    value: &Expression<'_>,
    ctx: &Ctx<'_, '_>,
) -> Result<()> {
    let value = shared::expression_value(value, &ctx.bindings);
    match name.as_deref() {
        Some("name") => options.name = shared::optional_string(value, ctx.source),
        Some("root") => options.root = shared::optional_string(value, ctx.source),
        Some("extends") => {
            options.extends = match crate::codebase::ts_source::unwrap_ts_wrappers(value) {
                Expression::BooleanLiteral(boolean) => Some(boolean.value),
                _ => None,
            };
        }
        Some("include") => {
            let include = shared::inferred_string_or_array(value, ctx.source, "include")?;
            if include.is_empty() {
                anyhow::bail!("expected string literal or string array for include");
            }
            options.include = Some(include);
        }
        Some("exclude") => {
            options.exclude = Some(shared::inferred_string_or_array(
                value, ctx.source, "exclude",
            )?);
        }
        Some("setupFiles") => {
            options.setup_files =
                Some(setup_dependencies(value, VitestSetupField::SetupFiles, ctx));
        }
        Some("globalSetup") => {
            options.global_setup = Some(setup_dependencies(
                value,
                VitestSetupField::GlobalSetup,
                ctx,
            ));
        }
        _ => {}
    }
    Ok(())
}
