use super::{
    import_bindings, named_export_object, objects, project_options_in, shared, sourced_reexport,
    top_level_function_bodies, Ctx, ImportBinding, Options, Scope,
};
use crate::ast;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn local_member_project_options(
    object: &str,
    member: &str,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
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
            Some(object) => project_options_in(object, ctx, scope),
            None => super::spread_project_options(expression, ctx, scope),
        };
    }
    imported_spread_member_project_options(map, member, ctx, scope)
}

pub(super) fn imported_member_project_options(
    import: &ImportBinding,
    member: &str,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
) -> Result<Option<Vec<Options>>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(None);
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(None);
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(None),
        Ok(source) => ast::with_program(&path, &source, |program, source| {
            exported_member_project_options(
                program,
                source,
                &path,
                ctx,
                &import.imported,
                member,
                scope,
            )
        })
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
    scope: Scope,
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
            return imported_member_project_options(&import, member, path, parent, scope);
        }
        if let Some(import) = super::imported_reexport(program, exported) {
            return imported_member_project_options(&import, member, path, parent, scope);
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
    project_options_in(object, &mut ctx, scope)
}

fn imported_spread_member_project_options(
    object: &ObjectExpression<'_>,
    member: &str,
    ctx: &mut Ctx<'_, '_>,
    scope: Scope,
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
                scope,
            )?
        } else {
            imported_member_project_options(&import, member, ctx.path, ctx, scope)?
        };
        if projects.is_some() {
            found = projects;
        }
    }
    Ok(found)
}
