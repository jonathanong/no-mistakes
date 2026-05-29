use super::{
    expression_options, import_bindings, imported_options, root_spreads, shared,
    top_level_function_bodies, Ctx, ImportBinding,
};
use crate::ast;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::test_config::playwright::Options;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program, Statement};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn namespace_member_options(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Expression::Identifier(object) = &member.object else {
        return Ok(Vec::new());
    };
    if let Some(import) = ctx.imports.get(object.name.as_str()).cloned() {
        if import.imported != "*" {
            return imported_member_options(&import, member.property.name.as_str(), ctx);
        }
        return imported_options(
            &ImportBinding {
                source: import.source,
                imported: member.property.name.to_string(),
            },
            ctx,
        );
    }
    if let Some(object) = ctx
        .bindings
        .get(object.name.as_str())
        .and_then(|expression| super::objects::expression_object(expression, &ctx.bindings))
    {
        match shared::property_expression_deep(object, member.property.name.as_str(), &ctx.bindings)
        {
            Some(expression) => expression_options(expression, ctx),
            None => imported_spread_member_options(object, member.property.name.as_str(), ctx),
        }
    } else {
        Ok(Vec::new())
    }
}

fn imported_spread_member_options(
    object: &ObjectExpression<'_>,
    member: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let mut found = Vec::new();
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
        let options = if import.imported == "*" {
            imported_options(
                &ImportBinding {
                    source: import.source,
                    imported: member.to_string(),
                },
                ctx,
            )?
        } else {
            imported_member_options(&import, member, ctx)?
        };
        if !options.is_empty() {
            found = options;
        }
    }
    Ok(found)
}

fn imported_member_options(
    import: &ImportBinding,
    member: &str,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    imported_member_options_from(import, member, ctx.path, ctx)
}

fn imported_options_from_base(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(Vec::new());
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(Vec::new());
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(Vec::new()),
        Ok(source) => ast::with_program(&path, &source, |program, source| {
            super::exports::exported_options(program, source, &path, ctx, &import.imported)
        })
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn imported_member_options_from(
    import: &ImportBinding,
    member: &str,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    let Some(path) = ctx.resolver.resolve(&import.source, base_path) else {
        return Ok(Vec::new());
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(Vec::new());
    }
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(Vec::new()),
        Ok(source) => ast::with_program(&path, &source, |program, source| {
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
) -> Result<Vec<Options>> {
    let bindings = shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        shared::default_export_object(program, &bindings)
            .or_else(|| root_spreads::named_export_object(program, exported, &bindings))
    } else {
        root_spreads::named_export_object(program, exported, &bindings)
    };
    let Some(object) = object else {
        if let Some(import) = root_spreads::sourced_reexport(program, exported) {
            return imported_member_options_from(&import, member, path, parent);
        }
        if let Some(import) = root_spreads::imported_reexport(program, exported) {
            return imported_member_options_from(&import, member, path, parent);
        }
        for statement in &program.body {
            let Statement::ExportAllDeclaration(export) = statement else {
                continue;
            };
            if export.export_kind.is_type()
                || export.exported.as_ref().map(|n| n.name()) != Some(exported.into())
            {
                continue;
            }
            let binding = ImportBinding {
                source: export.source.value.to_string(),
                imported: member.to_string(),
            };
            let options = imported_options_from_base(&binding, path, parent)?;
            if !options.is_empty() {
                return Ok(options);
            }
        }
        return Ok(Vec::new());
    };
    let Some(expression) = shared::property_expression_deep(object, member, &bindings) else {
        return Ok(Vec::new());
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
    expression_options(expression, &mut ctx)
}
