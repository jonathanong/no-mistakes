use super::{
    imported_reexport, named_export_object, sourced_reexport, top_level_function_bodies, Context,
    ExprMap, ImportBinding,
};
use crate::integration_tests::test_config::shared;
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, Program};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

type ExpressionObject =
    for<'a> fn(&'a Expression<'a>, &ExprMap<'a>) -> Option<&'a ObjectExpression<'a>>;
type ProjectOptions<T> = fn(&ObjectExpression<'_>, &mut Context<'_, '_>) -> Result<T>;
type ImportedOptions<T> = fn(&ImportBinding, &mut Context<'_, '_>) -> Result<Option<T>>;

pub(in crate::integration_tests::test_config) struct ObjectMemberOps<T> {
    pub expression_object: ExpressionObject,
    pub project_options: ProjectOptions<T>,
    pub imported_options: ImportedOptions<T>,
    pub import_bindings: fn(&Program<'_>) -> BTreeMap<String, ImportBinding>,
    pub default_identifier_reexport: bool,
}

pub(in crate::integration_tests::test_config) fn namespace_member_options<T>(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    ctx: &mut Context<'_, '_>,
    ops: &ObjectMemberOps<T>,
) -> Result<Option<T>> {
    let Expression::Identifier(object) = &member.object else {
        return Ok(None);
    };
    if let Some(import) = ctx.imports.get(object.name.as_str()).cloned() {
        if import.imported != "*" {
            return imported_member_options(
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
    let Some(object) = ctx
        .bindings
        .get(object.name.as_str())
        .and_then(|expression| (ops.expression_object)(expression, &ctx.bindings))
    else {
        return Ok(None);
    };
    let Some(expression) =
        shared::property_expression_deep(object, member.property.name.as_str(), &ctx.bindings)
    else {
        return Ok(None);
    };
    let Some(object) = (ops.expression_object)(expression, &ctx.bindings) else {
        return Ok(None);
    };
    (ops.project_options)(object, ctx).map(Some)
}

fn imported_member_options<T>(
    import: &ImportBinding,
    member: &str,
    base_path: &Path,
    ctx: &mut Context<'_, '_>,
    ops: &ObjectMemberOps<T>,
) -> Result<Option<T>> {
    super::with_imported_context(import, base_path, ctx, |program, source, path, ctx| {
        exported_member_options(
            program,
            source,
            import.imported.as_str(),
            member,
            path,
            ctx,
            ops,
        )
    })
}

fn exported_member_options<T>(
    program: &Program<'_>,
    source: &str,
    exported: &str,
    member: &str,
    path: &Path,
    parent: &mut Context<'_, '_>,
    ops: &ObjectMemberOps<T>,
) -> Result<Option<T>> {
    let bindings = shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        shared::default_export_object(program, &bindings)
            .or_else(|| named_export_object(program, exported, &bindings, ops.expression_object))
    } else {
        named_export_object(program, exported, &bindings, ops.expression_object)
    };
    let Some(object) = object else {
        if let Some(import) = sourced_reexport(program, exported) {
            return imported_member_options(&import, member, path, parent, ops);
        }
        if let Some(import) = imported_reexport(
            program,
            exported,
            &(ops.import_bindings)(program),
            ops.default_identifier_reexport,
        ) {
            return imported_member_options(&import, member, path, parent, ops);
        }
        return Ok(None);
    };
    let Some(expression) = shared::property_expression_deep(object, member, &bindings) else {
        return Ok(None);
    };
    let Some(object) = (ops.expression_object)(expression, &bindings) else {
        return Ok(None);
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
    (ops.project_options)(object, &mut ctx).map(Some)
}
