use super::fragments::Fragment;
use super::modules::module_file;
use super::resolve::FileCtx;
use super::tags::{imported, is_fragment_binding};
use super::text::{declarator_text, import_alias, whole_quoted};
use super::visible::visible_all;
use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueBinding, ValueExpression};
use std::path::PathBuf;

pub(super) fn namespace_of(ctx: &FileCtx<'_>, name: &str, depth: u8) -> Option<PathBuf> {
    if depth > 4 {
        return None;
    }
    if let Some(module) = callback_param(ctx, name) {
        return Some(module);
    }
    if let Some(module) = imported_namespace(ctx, name) {
        return Some(module);
    }
    if !locally_bound(ctx, name) {
        if let Some(alias) = super::fact_reads::equals_alias(ctx.ts, name) {
            return namespace_of(ctx, &alias, depth + 1);
        }
        if let Some(alias) = import_alias(ctx.source, name) {
            return namespace_of(ctx, alias, depth + 1);
        }
    }
    let binding = binding_named(ctx, name)?;
    if is_fragment_binding(ctx.ts, ctx.tags, binding) {
        return None;
    }
    match &binding.value {
        ValueExpression::Named { name: inner, .. } => namespace_of(ctx, inner, depth + 1),
        other => expr_module(ctx, other, depth + 1).or_else(|| initializer_module(ctx, binding)),
    }
}

fn initializer_module(ctx: &FileCtx<'_>, binding: &ValueBinding) -> Option<PathBuf> {
    let (start, _, window) = initializer_window(ctx, binding)?;
    if !super::fact_reads::window_loads_module(ctx.ts, window, start) {
        return None;
    }
    let spec = super::fact_reads::depth_zero_module_specifier(ctx.ts, window, start)?;
    module_file(ctx.graph, ctx.facts, ctx.catalog, ctx.file, &spec)
}

fn initializer_window<'a>(
    ctx: &'a FileCtx<'_>,
    binding: &ValueBinding,
) -> Option<(u32, u32, &'a str)> {
    match &binding.value {
        ValueExpression::Call(start, end) => Some((
            *start,
            *end,
            ctx.source.get(*start as usize..*end as usize)?,
        )),
        _ => {
            let rest = ctx.source.get(binding.declared_at as usize..)?;
            let window = declarator_text(rest);
            let end = binding
                .declared_at
                .checked_add(u32::try_from(window.len()).ok()?)?;
            Some((binding.declared_at, end, window))
        }
    }
}

pub(super) fn binding_named<'a>(ctx: &'a FileCtx<'_>, name: &str) -> Option<&'a ValueBinding> {
    if let Some(owner) = ctx.context {
        if let Some(binding) = ctx
            .ts
            .value_flow
            .bindings
            .iter()
            .rev()
            .find(|binding| binding.owner == Some(owner) && binding.name == name)
        {
            return Some(binding);
        }
    }
    ctx.ts
        .value_flow
        .bindings
        .iter()
        .rev()
        .find(|binding| binding.owner.is_none() && binding.name == name)
}

pub(super) fn expr_module(ctx: &FileCtx<'_>, expr: &ValueExpression, depth: u8) -> Option<PathBuf> {
    if depth > 4 {
        return None;
    }
    match expr {
        ValueExpression::Named { name, .. } => namespace_of(ctx, name, depth),
        ValueExpression::Awaited(inner) => expr_module(ctx, inner, depth + 1),
        ValueExpression::Call(start, end) => import_module(ctx, *start, *end),
        _ => None,
    }
}

pub(super) fn import_fragments(ctx: &FileCtx<'_>, start: u32, end: u32) -> Vec<Fragment> {
    import_module(ctx, start, end)
        .map(|module| visible_all(ctx.catalog, ctx.graph, ctx.facts, &module, &mut Vec::new()))
        .unwrap_or_default()
}

fn imported_namespace(ctx: &FileCtx<'_>, name: &str) -> Option<PathBuf> {
    if locally_bound(ctx, name) {
        return None;
    }
    import_namespace(ctx, name).or_else(|| import_equals(ctx, name))
}

pub(super) fn locally_bound(ctx: &FileCtx<'_>, name: &str) -> bool {
    let Some(owner) = ctx.context else {
        return false;
    };
    ctx.ts
        .value_flow
        .parameters
        .iter()
        .any(|parameter| parameter.owner == owner && parameter.name == name)
        || ctx
            .ts
            .value_flow
            .bindings
            .iter()
            .any(|binding| binding.owner == Some(owner) && binding.name == name)
}

fn import_namespace(ctx: &FileCtx<'_>, name: &str) -> Option<PathBuf> {
    let binding = imported(ctx.ts, name)?;
    if binding.kind != ImportedBindingKind::Namespace {
        return None;
    }
    module_file(
        ctx.graph,
        ctx.facts,
        ctx.catalog,
        ctx.file,
        &binding.specifier,
    )
}

fn callback_param(ctx: &FileCtx<'_>, name: &str) -> Option<PathBuf> {
    let context = ctx.context?;
    let module = ctx.callbacks.get(&context)?.clone();
    ctx.ts
        .value_flow
        .parameters
        .iter()
        .any(|parameter| {
            parameter.owner == context && parameter.index == 0 && parameter.name == name
        })
        .then_some(module)
}

fn import_equals(ctx: &FileCtx<'_>, name: &str) -> Option<PathBuf> {
    if let Some(spec) = super::fact_reads::require_specifier(ctx.ts, name) {
        return module_file(ctx.graph, ctx.facts, ctx.catalog, ctx.file, spec);
    }
    let marker = format!("import {name} = require(");
    let mut rest = ctx.source;
    while let Some(index) = rest.find(&marker) {
        let after = &rest[index + marker.len()..];
        if let Some(spec) = whole_quoted(after) {
            if ctx
                .ts
                .namespaces
                .opaque_specifiers
                .iter()
                .any(|item| item == spec)
            {
                return module_file(ctx.graph, ctx.facts, ctx.catalog, ctx.file, spec);
            }
        }
        rest = &rest[index + marker.len()..];
    }
    None
}

fn import_module(ctx: &FileCtx<'_>, start: u32, end: u32) -> Option<PathBuf> {
    let window = ctx.source.get(start as usize..end as usize).unwrap_or("");
    let spec = super::fact_reads::depth_zero_module_specifier(ctx.ts, window, start)?;
    module_file(ctx.graph, ctx.facts, ctx.catalog, ctx.file, &spec)
}
