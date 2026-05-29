use super::{
    import_bindings, root_spreads, shared, top_level_function_bodies, Ctx, ExprMap, ImportBinding,
    Options,
};
use crate::ast;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use anyhow::Result;
use exports::exported_star_options;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, Program};
use std::collections::BTreeSet;
use std::path::Path;

mod calls;
mod exports;
mod members;

pub(super) fn project_options(
    object: &ObjectExpression<'_>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Options> {
    parse_options(object, ctx)
}

pub(super) fn expression_object<'a>(
    expression: &'a Expression<'a>,
    bindings: &ExprMap<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let mut seen = BTreeSet::new();
    shared::expression_config_object(expression, bindings, &mut seen)
}

fn parse_options(object: &ObjectExpression<'_>, ctx: &mut Ctx<'_, '_>) -> Result<Options> {
    let mut options = Options::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed || property.method {
                    continue;
                }
                let name = shared::property_key_name(&property.key);
                if name.as_deref() == Some("test") {
                    if let Some(test_options) = expression_object_options(&property.value, ctx)? {
                        options.name = None;
                        options.include = None;
                        options.exclude = None;
                        merge_options(&mut options, test_options);
                    }
                    continue;
                }
                merge_property(&mut options, name, &property.value, ctx)?;
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Some(imported) = spread_options(&spread.argument, ctx)? {
                    merge_options(&mut options, imported);
                }
            }
        }
    }
    Ok(options)
}

fn merge_property(
    options: &mut Options,
    name: Option<String>,
    value: &Expression<'_>,
    ctx: &Ctx<'_, '_>,
) -> Result<()> {
    let value = shared::expression_value(value, &ctx.bindings);
    match name.as_deref() {
        Some("name") => options.name = shared::optional_string(value, ctx.source),
        Some("root") => options.root = shared::optional_string(value, ctx.source),
        Some("include") => {
            let include = shared::inferred_string_or_array(value, ctx.source, "include")?;
            if include.is_empty() {
                anyhow::bail!("expected string literal or string array for include");
            }
            options.include = Some(include);
        }
        Some("exclude") => {
            options.exclude = Some(shared::inferred_string_or_array(
                value, ctx.source, "exclude",
            )?);
        }
        _ => {}
    }
    Ok(())
}

fn spread_options(expression: &Expression<'_>, ctx: &mut Ctx<'_, '_>) -> Result<Option<Options>> {
    expression_object_options(expression, ctx)
}

pub(super) fn expression_object_options(
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
            calls::call_object_options(&call.callee, ctx)
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

fn imported_options(import: &ImportBinding, ctx: &mut Ctx<'_, '_>) -> Result<Option<Options>> {
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
    let result = match std::fs::read_to_string(&path) {
        Err(_) => Ok(None),
        Ok(source) => ast::with_program(&path, &source, |program, source| {
            exported_options(program, source, import.imported.as_str(), &path, ctx)
        })
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
    let bindings = shared::top_level_object_bindings(program);
    let object = if exported == "default" {
        shared::default_export_object(program, &bindings)
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

fn merge_options(base: &mut Options, next: Options) {
    if next.name.is_some() {
        base.name = next.name;
    }
    if next.root.is_some() {
        base.root = next.root;
    }
    if next.include.is_some() {
        base.include = next.include;
    }
    if next.exclude.is_some() {
        base.exclude = next.exclude;
    }
}
