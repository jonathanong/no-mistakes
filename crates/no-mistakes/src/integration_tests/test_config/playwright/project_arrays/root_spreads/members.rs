use super::{
    import_bindings, named_export_object, objects, project_options, shared, sourced_reexport,
    top_level_function_bodies, Ctx, ImportBinding, Options,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn local_member_project_options(
    object: &str,
    member: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Vec<Options>>> {
    let Some(map) = ctx
        .bindings
        .get(object)
        .and_then(|expression| objects::expression_object(expression, &ctx.bindings))
    else {
        return Ok(None);
    };
    if let Some(expression) = shared::property_expression_deep(map, member, &ctx.bindings) {
        return match objects::expression_object(expression, &ctx.bindings) {
            Some(object) => project_options(object, ctx),
            None => Ok(None),
        };
    }
    imported_spread_member_project_options(map, member, ctx)
}

pub(super) fn imported_member_project_options(
    import: &ImportBinding,
    member: &str,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Vec<Options>>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(None);
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(None);
    }
    let result = match crate::integration_tests::runner_config::read_request_source(&path) {
        Err(_) => Ok(None),
        Ok(source) => crate::integration_tests::runner_config::with_program(
            &path,
            &source,
            |program, source| {
                exported_member_project_options(
                    program,
                    source,
                    &path,
                    ctx,
                    &import.imported,
                    member,
                )
            },
        )
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn exported_member_project_options(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
    exported: &str,
    member: &str,
) -> Result<Option<Vec<Options>>> {
    let bindings = shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        shared::default_export_object(program, &bindings)
            .or_else(|| named_export_object(program, exported, &bindings))
    } else {
        named_export_object(program, exported, &bindings)
    };
    let Some(object) = object else {
        if let Some(import) = sourced_reexport(program, exported) {
            return imported_member_project_options(&import, member, path, parent);
        }
        if let Some(import) = super::imported_reexport(program, exported) {
            return imported_member_project_options(&import, member, path, parent);
        }
        return Ok(None);
    };
    let Some(expression) = shared::property_expression_deep(object, member, &bindings) else {
        return Ok(None);
    };
    let Some(object) = objects::expression_object(expression, &bindings) else {
        return Ok(None);
    };
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut ctx = Ctx {
        source,
        bindings,
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver: parent.resolver,
        path,
        seen: parent.seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    project_options(object, &mut ctx)
}

fn imported_spread_member_project_options(
    object: &ObjectExpression<'_>,
    member: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Vec<Options>>> {
    let mut found = None;
    for property in &object.properties {
        let ObjectPropertyKind::SpreadProperty(spread) = property else {
            continue;
        };
        let Expression::Identifier(identifier) = unwrap_ts_wrappers(&spread.argument) else {
            continue;
        };
        let Some(import) = ctx.imports.get(identifier.name.as_str()).cloned() else {
            continue;
        };
        let projects = if import.imported == "*" {
            super::imported_project_options(
                &ImportBinding {
                    source: import.source,
                    imported: member.to_string(),
                },
                ctx.path,
                ctx,
            )?
        } else {
            imported_member_project_options(&import, member, ctx.path, ctx)?
        };
        if projects.is_some() {
            found = projects;
        }
    }
    Ok(found)
}
