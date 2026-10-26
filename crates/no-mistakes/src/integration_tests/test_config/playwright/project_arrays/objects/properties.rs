use super::{shared, Ctx, Options};
use crate::integration_tests::test_config::deadlines;
use anyhow::Result;
use oxc_ast::ast::Expression;

pub(super) fn merge_property(
    options: &mut Options,
    name: Option<&str>,
    value: &Expression<'_>,
    property_start: u32,
    ctx: &Ctx<'_, '_>,
) -> Result<()> {
    if name == Some("timeout") {
        options.deadlines.case = Some(deadlines::declaration(value, ctx.path));
    }
    let value = shared::expression_value(value, &ctx.bindings);
    match name {
        Some("name") => options.name = shared::optional_string(value, ctx.source),
        Some("testDir") => {
            options.test_dir = Some(shared::required_string(value, ctx.source, "testDir")?);
        }
        Some("testMatch") => {
            options.test_match = Some(crate::playwright::test_patterns::Patterns::parse(
                value,
                ctx.source,
                ctx.path,
                "testMatch",
                property_start,
            ));
        }
        Some("testIgnore") => {
            options.test_ignore = Some(crate::playwright::test_patterns::Patterns::parse(
                value,
                ctx.source,
                ctx.path,
                "testIgnore",
                property_start,
            ));
        }
        _ => {}
    }
    Ok(())
}
