pub(in crate::integration_tests::test_config) use super::member_ops::MemberOps;
use super::{
    imported_reexport, named_export_object, sourced_reexport, top_level_function_bodies, Context,
    ImportBinding,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::integration_tests::test_config::shared;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program, Statement};
use std::collections::BTreeSet;
use std::path::Path;

pub(in crate::integration_tests::test_config) fn namespace_member_options<T>(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Context<'_, '_>,
    ops: &MemberOps<T>,
) -> Result<Vec<T>> {
    let Expression::Identifier(object) = &member.object else {
        return Ok(Vec::new());
    };
    if let Some(import) = ctx.imports.get(object.name.as_str()).cloned() {
        if import.imported != "*" {
            return imported_member_options_from(
                &import,
                member.property.name.as_str(),
                ctx.path,
                ctx,
                ops,
            );
        }
        return (ops.imported_options)(
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
        .and_then(|expression| (ops.expression_object)(expression, &ctx.bindings))
    {
        match shared::property_expression_deep(object, member.property.name.as_str(), &ctx.bindings)
        {
            Some(expression) => (ops.expression_options)(expression, ctx),
            None => imported_spread_member_options(object, member.property.name.as_str(), ctx, ops),
        }
    } else {
        Ok(Vec::new())
    }
}

fn imported_spread_member_options<T>(
    object: &ObjectExpression<'_>,
    member: &str,
    ctx: &mut Context<'_, '_>,
    ops: &MemberOps<T>,
) -> Result<Vec<T>> {
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
            (ops.imported_options)(
                &ImportBinding {
                    source: import.source,
                    imported: member.to_string(),
                },
                ctx,
            )?
        } else {
            imported_member_options_from(&import, member, ctx.path, ctx, ops)?
        };
        if !options.is_empty() {
            found = options;
        }
    }
    Ok(found)
}

fn imported_options_from_base<T>(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Context<'_, '_>,
    ops: &MemberOps<T>,
) -> Result<Vec<T>> {
    crate::integration_tests::test_config::shared_project_arrays::with_imported_context(
        import,
        base_path,
        ctx,
        |program, source, path, ctx| {
            (ops.exported_options)(program, source, path, ctx, &import.imported)
        },
    )
}

pub(in crate::integration_tests::test_config) fn imported_member_options_from<T>(
    import: &ImportBinding,
    member: &str,
    base_path: &Path,
    ctx: &mut Context<'_, '_>,
    ops: &MemberOps<T>,
) -> Result<Vec<T>> {
    crate::integration_tests::test_config::shared_project_arrays::with_imported_context(
        import,
        base_path,
        ctx,
        |program, source, path, ctx| {
            exported_member_options(
                program,
                source,
                import.imported.as_str(),
                member,
                path,
                ctx,
                ops,
            )
        },
    )
}

fn exported_member_options<T>(
    program: &Program<'_>,
    source: &str,
    exported: &str,
    member: &str,
    path: &Path,
    parent: &mut Context<'_, '_>,
    ops: &MemberOps<T>,
) -> Result<Vec<T>> {
    let bindings = shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        shared::default_export_object(program, &bindings)
            .or_else(|| named_export_object(program, exported, &bindings, ops.expression_object))
    } else {
        named_export_object(program, exported, &bindings, ops.expression_object)
    };
    let Some(object) = object else {
        if let Some(import) = sourced_reexport(program, exported) {
            return imported_member_options_from(&import, member, path, parent, ops);
        }
        if let Some(import) = imported_reexport(
            program,
            exported,
            &(ops.import_bindings)(program),
            ops.default_identifier_reexport,
        ) {
            return imported_member_options_from(&import, member, path, parent, ops);
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
                source: crate::ast::js_literal(&export.source).into_owned(),
                imported: member.to_string(),
            };
            let options = imported_options_from_base(&binding, path, parent, ops)?;
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
    let mut ctx = Context {
        is_test_object: parent.is_test_object,
        source,
        bindings,
        functions: top_level_function_bodies(program),
        imports: (ops.import_bindings)(program),
        resolver: parent.resolver,
        path,
        seen: parent.seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    (ops.expression_options)(expression, &mut ctx)
}
