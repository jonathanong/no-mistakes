use super::{Ctx, Options};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

pub(super) fn spread_project_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> anyhow::Result<Option<Vec<Options>>> {
    let mut options = super::spread_project_options_inner(expression, ctx)?;
    if !matches!(
        unwrap_ts_wrappers(expression),
        Expression::ObjectExpression(_)
    ) {
        if let Some(options) = &mut options {
            for option in options {
                crate::integration_tests::test_config::deadlines::unproved_case(
                    &mut option.deadlines,
                    ctx.path,
                    expression.span(),
                );
            }
        }
    }
    Ok(options)
}
