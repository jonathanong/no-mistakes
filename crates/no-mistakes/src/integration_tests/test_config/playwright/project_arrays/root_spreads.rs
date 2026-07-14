use super::{
    expression_options, import_bindings, objects, shared, top_level_function_bodies, Ctx,
    ImportBinding, Options,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program};
use std::collections::BTreeSet;
use std::path::Path;

mod calls;
mod exports;
mod members;
pub(in crate::integration_tests::test_config::playwright::project_arrays) use exports::{
    imported_reexport, named_export_object, sourced_reexport,
};

pub(super) fn project_options(
    object: &ObjectExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Vec<Options>>> {
    let mut found = None;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed || property.method {
                    continue;
                }
                if shared::property_key_name(&property.key).as_deref() == Some("projects") {
                    found = Some(expression_options(&property.value, ctx)?);
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(projects) = spread_project_options(&spread.argument, ctx)? {
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
) -> Result<Option<Vec<Options>>> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if let Some(import) = ctx.imports.get(name).cloned() {
                return imported_project_options(&import, ctx.path, ctx);
            }
            if !ctx.object_seen.insert(name.to_string()) {
                return Ok(None);
            }
            let result = match ctx
                .bindings
                .get(name)
                .and_then(|expression| objects::expression_object(expression, &ctx.bindings))
            {
                Some(object) => project_options(object, ctx),
                None => Ok(None),
            };
            ctx.object_seen.remove(name);
            result
        }
        Expression::StaticMemberExpression(member) => namespace_member_project_options(member, ctx),
        Expression::CallExpression(call) if call.arguments.is_empty() => {
            calls::call_project_options(&call.callee, ctx)
        }
        _ => Ok(None),
    }
}

fn namespace_member_project_options(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Vec<Options>>> {
    let Expression::Identifier(object) = &member.object else {
        return Ok(None);
    };
    let Some(import) = ctx.imports.get(object.name.as_str()).cloned() else {
        return members::local_member_project_options(
            object.name.as_str(),
            member.property.name.as_str(),
            ctx,
        );
    };
    if import.imported != "*" {
        return members::imported_member_project_options(
            &import,
            member.property.name.as_str(),
            ctx.path,
            ctx,
        );
    }
    imported_project_options(
        &ImportBinding {
            source: import.source,
            imported: member.property.name.to_string(),
        },
        ctx.path,
        ctx,
    )
}

fn imported_project_options(
    import: &ImportBinding,
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
                exported_project_options(program, source, import.imported.as_str(), &path, ctx)
            },
        )
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
            return imported_project_options(&import, path, parent);
        }
        if let Some(import) = imported_reexport(program, exported) {
            return imported_project_options(&import, path, parent);
        }
        for statement in &program.body {
            let oxc_ast::ast::Statement::ExportAllDeclaration(export) = statement else {
                continue;
            };
            if export.export_kind.is_type() || export.exported.is_some() {
                continue;
            }
            let binding = ImportBinding {
                source: export.source.value.to_string(),
                imported: exported.to_string(),
            };
            if let Some(result) = imported_project_options(&binding, path, parent)? {
                return Ok(Some(result));
            }
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
    project_options(object, &mut ctx)
}
