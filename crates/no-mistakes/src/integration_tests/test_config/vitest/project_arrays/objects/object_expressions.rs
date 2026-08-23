use super::super::{import_bindings, root_spreads, top_level_function_bodies, ImportBinding};
use super::exports::exported_star_options;
use super::{expression_object, members, project_options, Ctx, Options};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use oxc_ast::ast::{Expression, Program};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn spread_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    expression_object_options(expression, ctx)
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn expression_object_options(
    expression: &Expression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    match unwrap_ts_wrappers(expression) {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if let Some(import) = ctx.imports.get(name).cloned() {
                return imported_options(&import, ctx);
            }
            if !ctx.object_seen.insert(name.to_string()) {
                return Ok(None);
            }
            let result = match ctx
                .bindings
                .get(name)
                .and_then(|expression| expression_object(expression, &ctx.bindings))
            {
                Some(object) => project_options(object, ctx).map(Some),
                None => Ok(None),
            };
            ctx.object_seen.remove(name);
            result
        }
        Expression::StaticMemberExpression(member) => {
            members::namespace_member_options(member, ctx)
        }
        Expression::CallExpression(call) if call.arguments.is_empty() => {
            super::calls::call_object_options(&call.callee, ctx)
        }
        Expression::CallExpression(_) => Ok(None),
        _ => {
            let Some(object) = expression_object(expression, &ctx.bindings) else {
                return Ok(None);
            };
            project_options(object, ctx).map(Some)
        }
    }
}

pub(super) fn imported_options(
    import: &ImportBinding,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    imported_options_from(import, ctx.path, ctx)
}

fn imported_options_from(
    import: &ImportBinding,
    base_path: &Path,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
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
                exported_options(program, source, import.imported.as_str(), &path, ctx)
            },
        )
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    result
}

fn exported_options(
    program: &Program<'_>,
    source: &str,
    exported: &str,
    path: &Path,
    parent: &mut Ctx<'_, '_>,
) -> Result<Option<Options>> {
    let bindings = super::shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        super::shared::default_export_object(program, &bindings)
            .or_else(|| root_spreads::named_export_object(program, exported, &bindings))
    } else {
        root_spreads::named_export_object(program, exported, &bindings)
    };
    let Some(object) = object else {
        if let Some(import) = root_spreads::sourced_reexport(program, exported) {
            return imported_options_from(&import, path, parent);
        }
        if let Some(import) = root_spreads::imported_reexport(program, exported) {
            return imported_options_from(&import, path, parent);
        }
        return exported_star_options(program, exported, parent);
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
