use super::{
    expression_options, import_bindings, objects, shared, top_level_function_bodies, Ctx,
    ImportBinding, Options,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
mod calls;
mod exports;
mod imports;
mod members;
pub(in crate::integration_tests::test_config::vitest::project_arrays) use exports::{
    imported_reexport, named_export_object, sourced_reexport, star_barrel_sources,
};
use imports::imported_project_options;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Root,
    Test,
}

pub(super) fn project_options(
    object: &ObjectExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Vec<Options>>> {
    project_options_in(object, ctx, Scope::Root)
}

fn project_options_in(
    object: &ObjectExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let mut found = None;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed || property.method {
                    continue;
                }
                match shared::property_key_name(&property.key).as_deref() {
                    Some("test") if scope == Scope::Root => {
                        if let Some(test_object) =
                            objects::expression_object(&property.value, &ctx.bindings)
                        {
                            found = project_options_in(test_object, ctx, Scope::Test)?;
                        } else if let Some(projects) =
                            spread_project_options(&property.value, ctx, Scope::Test)?
                        {
                            found = Some(projects);
                        } else {
                            found = None;
                        }
                    }
                    Some("projects") if scope == Scope::Test => {
                        found = Some(expression_options(&property.value, ctx)?);
                    }
                    _ => {}
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(projects) = spread_project_options(&spread.argument, ctx, scope)? {
                    found = Some(projects);
                }
            }
        }
    }
    Ok(found)
}

fn spread_project_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if let Some(import) = ctx.imports.get(name).cloned() {
                return imported_project_options(&import, ctx.path, ctx, scope);
            }
            if !ctx.object_seen.insert(name.to_string()) {
                return Ok(None);
            }
            let result = match ctx
                .bindings
                .get(name)
                .and_then(|expression| objects::expression_object(expression, &ctx.bindings))
            {
                Some(object) => project_options_in(object, ctx, scope),
                None => Ok(None),
            };
            ctx.object_seen.remove(name);
            result
        }
        Expression::StaticMemberExpression(member) => {
            namespace_member_project_options(member, ctx, scope)
        }
        Expression::CallExpression(call) if call.arguments.is_empty() => {
            calls::call_project_options(&call.callee, ctx, scope)
        }
        _ => Ok(None),
    }
}

fn namespace_member_project_options(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let Expression::Identifier(object) = &member.object else {
        return Ok(None);
    };
    let Some(import) = ctx.imports.get(object.name.as_str()).cloned() else {
        return members::local_member_project_options(
            object.name.as_str(),
            member.property.name.as_str(),
            ctx,
            scope,
        );
    };
    if import.imported != "*" {
        return members::imported_member_project_options(
            &import,
            member.property.name.as_str(),
            ctx.path,
            ctx,
            scope,
        );
    }
    let binding = ImportBinding {
        source: import.source,
        imported: member.property.name.to_string(),
    };
    imported_project_options(&binding, ctx.path, ctx, scope)
}
