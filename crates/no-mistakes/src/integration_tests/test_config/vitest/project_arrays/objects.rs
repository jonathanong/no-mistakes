use super::{shared, Ctx, ExprMap, ImportBinding, Options};
use crate::integration_tests::test_config::vitest::Extends;
use crate::integration_tests::types::VitestSetupField;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use std::collections::BTreeSet;

mod calls;
mod config_extends;
mod dynamic_triggers;
mod exports;
mod members;
mod merge;
mod object_expressions;
mod setup_dependencies;
mod setup_imports;
mod static_members;

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
            if !property.computed && !property.method
                && shared::property_key_name(&property.key).as_deref() == Some("test"))
    });
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed || property.method {
                    continue;
                }
                let name = shared::property_key_name(&property.key);
                if name.as_deref() == Some("test") {
                    if let Some(mut test_options) = expression_object_options(&property.value, ctx)?
                    {
                        options.name = None;
                        options.include = None;
                        options.exclude = None;
                        options.setup_files = None;
                        options.global_setup = None;
                        options.setup_files_cleared = false;
                        options.global_setup_cleared = false;
                        test_options.nested_test_scope = true;
                        merge_options(&mut options, test_options);
                    }
                    continue;
                }
                let nested_test_scope = nested_test || options.nested_test_scope;
                merge_property(&mut options, name, &property.value, nested_test_scope, ctx)?;
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(imported) = spread_options(&spread.argument, ctx)? {
                    merge_options(&mut options, imported);
                }
            }
        }
    }
    resolve_config_extends(&mut options, ctx)?;
    Ok(options)
}

fn merge_property(
    options: &mut Options,
    name: Option<String>,
    value: &Expression<'_>,
    nested_test: bool,
    ctx: &mut Ctx<'_, '_>,
) -> Result<()> {
    let resolved = shared::expression_value(value, &ctx.bindings);
    match name.as_deref() {
        Some("name") => options.name = static_project_name(resolved, ctx.source),
        Some("root") => options.root = shared::optional_string(resolved, ctx.source),
        Some("extends") => {
            options.extends = match crate::codebase::ts_source::unwrap_ts_wrappers(resolved) {
                Expression::BooleanLiteral(boolean) => Some(if boolean.value {
                    Extends::True
                } else {
                    Extends::False
                }),
                _ => shared::optional_string(resolved, ctx.source).map(Extends::Config),
            };
        }
        Some("include") => {
            let include = shared::inferred_string_or_array(resolved, ctx.source, "include")?;
            if include.is_empty() {
                anyhow::bail!("expected string literal or string array for include");
            }
            options.include = Some(include);
        }
        Some("exclude") => {
            options.exclude = Some(shared::inferred_string_or_array(
                resolved, ctx.source, "exclude",
            )?);
        }
        Some("setupFiles") if !nested_test => {
            let setups = setup_dependencies(value, VitestSetupField::SetupFiles, ctx);
            options.setup_files_cleared = setups.is_empty();
            options.setup_files = Some(setups);
        }
        Some("globalSetup") if !nested_test => {
            let setups = setup_dependencies(value, VitestSetupField::GlobalSetup, ctx);
            options.global_setup_cleared = setups.is_empty();
            options.global_setup = Some(setups);
        }
        _ => {}
    }
    Ok(())
}

fn static_project_name(value: &Expression<'_>, source: &str) -> Option<String> {
    shared::optional_string(value, source).or_else(|| {
        let Expression::ObjectExpression(object) =
            crate::codebase::ts_source::unwrap_ts_wrappers(value)
        else {
            return None;
        };
        object
            .properties
            .iter()
            .find_map(|property| match property {
                ObjectPropertyKind::ObjectProperty(property)
                    if !property.computed
                        && !property.method
                        && shared::property_key_name(&property.key).as_deref() == Some("label") =>
                {
                    shared::optional_string(&property.value, source)
                }
                _ => None,
            })
    })
}
