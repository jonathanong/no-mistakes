use super::fragments::Catalog;
use super::modules::import_hit_fragments;
use super::names::initializer_module;
use super::resolve::{binding_fragments, FileCtx};
use super::source_text::file_text;
use super::tags::binding_is_tag;
use super::visible::{visible, visible_all};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{ValueBinding, ValueExpression};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use std::path::Path;

pub(super) fn reaches_fragment(
    catalog: &Catalog<'_>,
    graph: Option<&DepGraph>,
    facts: &CheckFactMap,
    file: &Path,
) -> bool {
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    if !catalog.local_all(file).is_empty() {
        return true;
    }
    if ts
        .imported_bindings
        .iter()
        .chain(&ts.value_flow.commonjs_imported_bindings)
        .filter(|binding| !binding.is_type_only)
        .any(|binding| {
            let is_tag = binding_is_tag(ts, catalog.tags, &binding.local);
            !import_hit_fragments(catalog, graph, facts, file, binding, is_tag).is_empty()
        })
    {
        return true;
    }
    if (!ts.exported_bindings.is_empty() || !ts.star_reexport_specifiers.is_empty())
        && !visible_all(catalog, graph, facts, file, &mut Vec::new()).is_empty()
    {
        return true;
    }
    if ts.namespaces.literal_import_spans.is_empty() {
        return false;
    }
    let Some(source) = file_text(ts, catalog.sources, file) else {
        return false;
    };
    let callbacks = crate::fx::fx_map();
    let ctx = FileCtx {
        catalog,
        graph,
        facts,
        ts,
        file,
        source: &source,
        tags: catalog.tags,
        callbacks: &callbacks,
        context: None,
    };
    ts.value_flow
        .bindings
        .iter()
        .any(|binding| binding_reaches_fragment(&ctx, binding))
}

fn binding_reaches_fragment(ctx: &FileCtx<'_>, binding: &ValueBinding) -> bool {
    let ctx = FileCtx {
        context: binding.owner,
        ..*ctx
    };
    if !binding_fragments(&ctx, binding).is_empty() {
        return true;
    }
    let member = match &binding.value {
        ValueExpression::Awaited(_) => None,
        ValueExpression::Member(base, name)
            if matches!(base.as_ref(), ValueExpression::Awaited(_)) =>
        {
            Some(name.as_str())
        }
        _ => return false,
    };
    let Some(module) = initializer_module(&ctx, binding) else {
        return false;
    };
    let fragments = match member {
        Some(name) => visible(
            ctx.catalog,
            ctx.graph,
            ctx.facts,
            &module,
            name,
            &mut Vec::new(),
        ),
        None => visible_all(ctx.catalog, ctx.graph, ctx.facts, &module, &mut Vec::new()),
    };
    !fragments.is_empty()
}
