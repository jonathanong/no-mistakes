use super::super::{shared, Ctx, ImportBinding};
use super::setup_dependencies;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use oxc_ast::ast::{Expression, StaticMemberExpression};

pub(super) fn static_member_setup_dependencies(
    member: &StaticMemberExpression<'_>,
    field: VitestSetupField,
    ctx: &mut Ctx<'_, '_>,
) -> Option<Vec<VitestSetupDependency>> {
    let Expression::Identifier(object) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let name = object.name.to_string();
    let seen_key = format!("setup:{name}");
    if !ctx.object_seen.insert(seen_key.clone()) {
        return None;
    }
    let result = if let Some(import) = ctx.imports.get(&name).cloned() {
        if import.imported == "*" {
            super::setup_imports::imported_setup_dependencies(
                &ImportBinding {
                    source: import.source,
                    imported: member.property.name.to_string(),
                },
                field,
                ctx,
            )
        } else {
            super::setup_imports::imported_setup_member_dependencies(
                &import,
                member.property.name.as_str(),
                field,
                ctx,
            )
        }
    } else {
        ctx.bindings
            .get(&name)
            .and_then(|expression| super::expression_object(expression, &ctx.bindings))
            .and_then(|object| {
                shared::property_expression_deep(
                    object,
                    member.property.name.as_str(),
                    &ctx.bindings,
                )
            })
            .map(|expression| setup_dependencies(expression, field, ctx))
    };
    ctx.object_seen.remove(&seen_key);
    result
}
