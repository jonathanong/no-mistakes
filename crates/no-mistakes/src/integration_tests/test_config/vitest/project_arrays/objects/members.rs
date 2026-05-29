use super::super::{
    import_bindings, root_spreads, shared, top_level_function_bodies, Ctx, Options,
};
use super::{expression_object, imported_options, project_options};
use anyhow::Result;
use oxc_ast::ast::{Expression, Program};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn namespace_member_options(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Expression::Identifier(object) = &member.object else {
        return Ok(None);
    };
    if let Some(import) = ctx.imports.get(object.name.as_str()).cloned() {
        if import.imported != "*" {
            return imported_member_options(&import, member.property.name.as_str(), ctx.path, ctx);
        }
        return imported_options(
            &super::ImportBinding {
                source: import.source,
                imported: member.property.name.to_string(),
            },
            ctx,
        );
    }
    let Some(object) = ctx
        .bindings
        .get(object.name.as_str())
        .and_then(|expression| expression_object(expression, &ctx.bindings))
    else {
        return Ok(None);
    };
    let Some(expression) =
        shared::property_expression_deep(object, member.property.name.as_str(), &ctx.bindings)
    else {
        return Ok(None);
    };
    let Some(object) = expression_object(expression, &ctx.bindings) else {
        return Ok(None);
    };
    project_options(object, ctx).map(Some)
}

fn imported_member_options(
    import: &super::ImportBinding,
    member: &str,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(None);
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(None);
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(None),
        Ok(source) => crate::ast::with_program(&path, &source, |program, source| {
            exported_member_options(
                program,
                source,
                import.imported.as_str(),
                member,
                &path,
                ctx,
            )
        })
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn exported_member_options(
    program: &Program<'_>,
    source: &str,
    exported: &str,
    member: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let bindings = shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        shared::default_export_object(program, &bindings)
            .or_else(|| root_spreads::named_export_object(program, exported, &bindings))
    } else {
        root_spreads::named_export_object(program, exported, &bindings)
    };
    let Some(object) = object else {
        if let Some(import) = root_spreads::sourced_reexport(program, exported) {
            return imported_member_options(&import, member, path, parent);
        }
        if let Some(import) = root_spreads::imported_reexport(program, exported) {
            return imported_member_options(&import, member, path, parent);
        }
        return Ok(None);
    };
    let Some(expression) = shared::property_expression_deep(object, member, &bindings) else {
        return Ok(None);
    };
    let Some(object) = expression_object(expression, &bindings) else {
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
    project_options(object, &mut ctx).map(Some)
}
