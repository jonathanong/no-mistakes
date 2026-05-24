use super::{imported_options, Ctx, ImportBinding};
use crate::integration_tests::test_config::vitest::Options;
use anyhow::Result;
use oxc_ast::ast::Expression;

pub(super) fn namespace_member_options(
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
        &ImportBinding {
            source: import.source,
            imported: member.property.name.to_string(),
        },
        ctx,
    )
}
