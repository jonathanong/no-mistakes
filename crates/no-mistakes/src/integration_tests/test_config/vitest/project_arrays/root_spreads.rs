use super::{
    expression_options, import_bindings, objects, shared, top_level_function_bodies, Ctx,
    ImportBinding, Options,
};
use crate::ast;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program};
use std::collections::BTreeSet;
use std::path::Path;
mod calls;
mod exports;
mod members;
pub(in crate::integration_tests::test_config::vitest::project_arrays) use exports::{
    imported_reexport, named_export_object, sourced_reexport, star_barrel_sources,
};
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

fn imported_project_options(
    import: &ImportBinding,
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
            exported_project_options(program, source, import.imported.as_str(), &path, ctx, scope)
        })
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn exported_project_options(
    program: &Program<'_>,
    source: &str,
    exported: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
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
            return imported_project_options(&import, path, parent, scope);
        }
        if let Some(import) = imported_reexport(program, exported) {
            return imported_project_options(&import, path, parent, scope);
        }
        for source in star_barrel_sources(program) {
            let b = ImportBinding { source: source.to_string(), imported: exported.to_string() };
            if let Some(r) = imported_project_options(&b, path, parent, scope)? { return Ok(Some(r)); }
        }
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
