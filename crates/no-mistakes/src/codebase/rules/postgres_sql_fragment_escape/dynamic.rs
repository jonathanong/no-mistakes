use super::fragments::Fragment;
use super::modules::module_file;
use super::names::{expr_module, namespace_of};
use super::resolve::FileCtx;
use super::text::{import_call_specifier, quoted};
use super::visible::{visible, visible_all};
use crate::codebase::dependencies::extract::{ValueCall, ValueExpression};
use std::path::PathBuf;

pub(super) fn then_module(
    ctx: &FileCtx<'_>,
    call: &ValueCall,
    base: &ValueExpression,
) -> Option<PathBuf> {
    expr_module(ctx, base, 0).or_else(|| specifier_module(ctx, call.offset, call.end))
}

fn specifier_module(ctx: &FileCtx<'_>, start: u32, end: u32) -> Option<PathBuf> {
    let spec = import_call_specifier(source_slice(ctx.source, start, end)?)?;
    module_file(ctx.graph, ctx.facts, ctx.catalog, ctx.file, spec)
}

pub(super) fn dynamic_head(ctx: &FileCtx<'_>, call: &ValueCall) -> Vec<Fragment> {
    (|| {
        let text = source_slice(ctx.source, call.offset, call.end)?;
        let head = text.split_once('(')?.0.trim();
        let receiver = head.strip_suffix(".append")?.trim();
        Some(dynamic_computed(ctx, receiver, true))
    })()
    .unwrap_or_default()
}

pub(super) fn dynamic_argument(
    ctx: &FileCtx<'_>,
    call: &ValueCall,
    string_keys: bool,
) -> Vec<Fragment> {
    (|| {
        let text = source_slice(ctx.source, call.offset, call.end)?;
        let open = text.find('(')?;
        let close = text.rfind(')')?;
        if close <= open {
            return Some(Vec::new());
        }
        Some(dynamic_computed(
            ctx,
            text[open + 1..close].trim(),
            string_keys,
        ))
    })()
    .unwrap_or_default()
}

fn dynamic_computed(ctx: &FileCtx<'_>, text: &str, string_keys: bool) -> Vec<Fragment> {
    let Some((object, key)) = computed(text) else {
        return Vec::new();
    };
    let Some(module) = namespace_of(ctx, object, 0) else {
        return Vec::new();
    };
    if let Some(name) = string_key(key) {
        if !string_keys {
            return Vec::new();
        }
        return visible(
            ctx.catalog,
            ctx.graph,
            ctx.facts,
            &module,
            name,
            &mut Vec::new(),
        );
    }
    visible_all(ctx.catalog, ctx.graph, ctx.facts, &module, &mut Vec::new())
}

fn computed(text: &str) -> Option<(&str, &str)> {
    let text = text.trim().trim_end_matches(';');
    let open = text.rfind('[')?;
    if !text.ends_with(']') {
        return None;
    }
    let object = text[..open].trim().trim_end_matches('?');
    let key = text[open + 1..text.len() - 1].trim();
    is_ident(object).then_some((object, key))
}

fn string_key(key: &str) -> Option<&str> {
    let key = key.trim();
    let name = quoted(key)?;
    if name.is_empty() || name.len() + 2 != key.len() {
        return None;
    }
    Some(name)
}

fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn source_slice(source: &str, start: u32, end: u32) -> Option<&str> {
    source.get(start as usize..end as usize)
}
