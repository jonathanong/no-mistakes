use super::{deadlines, setup_dependencies, shared, Ctx, Extends, Options, VitestSetupField};
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectPropertyKind};
use oxc_span::GetSpan;

pub(super) fn merge_property(
    options: &mut Options,
    name: Option<String>,
    value: &Expression<'_>,
    nested_test: bool,
    ctx: &mut Ctx<'_, '_>,
) -> Result<()> {
    if !nested_test {
        match name.as_deref() {
            Some("testTimeout") => {
                options.deadlines.case = Some(deadlines::declaration(value, ctx.path))
            }
            Some("hookTimeout") => {
                options.deadlines.hook = Some(deadlines::declaration(value, ctx.path))
            }
            _ => {}
        }
    }
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
                _ => Some(
                    shared::optional_string(resolved, ctx.source)
                        .map(Extends::Config)
                        .unwrap_or(Extends::Unknown {
                            path: ctx.path.to_path_buf(),
                            span: Some((value.span().start, value.span().end)),
                        }),
                ),
            };
            if !ctx.is_test_object {
                options.deadline_extends = options.extends.clone();
            }
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
