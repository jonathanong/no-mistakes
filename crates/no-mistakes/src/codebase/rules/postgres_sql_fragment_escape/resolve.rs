use super::dynamic::{dynamic_argument, dynamic_head};
use super::fragments::{dedup, Catalog, Fragment};
use super::modules::import_hit_fragments;
use super::mutate::receiver_mutations;
use super::names::{binding_named, expr_module, import_fragments, locally_bound, namespace_of};
use super::tags::{binding_is_tag, imported, is_fragment_binding};
use super::text::import_equals_member;
use super::visible::{visible, visible_all};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    CallableId, ValueBinding, ValueCall, ValueExpression,
};
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(super) struct FileCtx<'a> {
    pub graph: &'a DepGraph,
    pub facts: &'a CheckFactMap,
    pub catalog: &'a Catalog<'a>,
    pub tags: &'a [TrustedSqlTag],
    pub file: &'a Path,
    pub ts: &'a TsFileFacts,
    pub source: &'a str,
    pub callbacks: &'a FxHashMap<CallableId, PathBuf>,
    pub context: Option<CallableId>,
}

pub(super) fn call_fragments(ctx: &FileCtx<'_>, call: &ValueCall) -> Vec<Fragment> {
    let ctx = FileCtx {
        context: call.context,
        ..*ctx
    };
    if let Some(base) = append_receiver(call) {
        let mut found = use_expr(&ctx, base, 0);
        if found.is_empty() {
            found.extend(dynamic_head(&ctx, call));
        }
        found.extend(dynamic_argument(&ctx, call, false));
        dedup(&mut found);
        return found;
    }
    let mut found = use_expr(&ctx, &call.callee, 0);
    found.extend(receiver_mutations(&ctx, call));
    for argument in call.arguments.iter() {
        if matches!(
            argument,
            ValueExpression::Call(_, _) | ValueExpression::Callable(_)
        ) {
            continue;
        }
        found.extend(use_expr(&ctx, argument, 0));
    }
    if call.arguments.len() == 1 && matches!(call.arguments.first(), Some(ValueExpression::Unknown))
    {
        found.extend(dynamic_argument(&ctx, call, true));
    }
    dedup(&mut found);
    found
}

pub(super) fn binding_fragments(ctx: &FileCtx<'_>, binding: &ValueBinding) -> Vec<Fragment> {
    if is_fragment_binding(ctx.ts, ctx.tags, binding) {
        return Vec::new();
    }
    use_expr(ctx, &binding.value, 0)
}

pub(super) fn name_fragments(ctx: &FileCtx<'_>, name: &str) -> Vec<Fragment> {
    use_name(ctx, name, 0)
}

fn append_receiver(call: &ValueCall) -> Option<&ValueExpression> {
    let ValueExpression::Member(base, name) = &call.callee else {
        return None;
    };
    if name == "append" && call.arguments.len() == 1 && call.spread_at.is_none() {
        Some(base.as_ref())
    } else {
        None
    }
}

pub(super) fn use_expr(ctx: &FileCtx<'_>, expr: &ValueExpression, depth: u8) -> Vec<Fragment> {
    if depth > 4 {
        return Vec::new();
    }
    match expr {
        ValueExpression::Named { name, .. } => use_name(ctx, name, depth),
        ValueExpression::Member(base, member) => member_of(ctx, base, member, depth),
        ValueExpression::Awaited(inner) => use_expr(ctx, inner, depth + 1),
        ValueExpression::Call(start, end) => import_fragments(ctx, *start, *end),
        ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right)
        | ValueExpression::Nullish(left, right) => {
            let mut found = use_expr(ctx, left, depth + 1);
            found.extend(use_expr(ctx, right, depth + 1));
            found
        }
        ValueExpression::Object(map, _) | ValueExpression::Array(map) => map
            .values()
            .flat_map(|value| use_expr(ctx, value, depth + 1))
            .collect(),
        ValueExpression::Derived(parts) => parts
            .iter()
            .flat_map(|value| use_expr(ctx, value, depth + 1))
            .collect(),
        _ => Vec::new(),
    }
}

fn use_name(ctx: &FileCtx<'_>, name: &str, depth: u8) -> Vec<Fragment> {
    if let Some(module) = namespace_of(ctx, name, depth) {
        return visible_all(ctx.catalog, ctx.graph, ctx.facts, &module, &mut Vec::new());
    }
    if !locally_bound(ctx, name) {
        if let Some(import) = imported(ctx.ts, name) {
            let mut found =
                import_hit_fragments(ctx.catalog, ctx.graph, ctx.facts, ctx.file, import);
            if binding_is_tag(ctx.ts, ctx.tags, name) {
                found.retain(|fragment| fragment.open_specifier.is_none());
            }
            return found;
        }
        let equals = super::fact_reads::equals_member(ctx.ts, name).or_else(|| {
            import_equals_member(ctx.source, name)
                .map(|(object, member)| (object.to_string(), member.to_string()))
        });
        if let Some((object, member)) = equals {
            if let Some(module) = namespace_of(ctx, &object, depth) {
                return visible(
                    ctx.catalog,
                    ctx.graph,
                    ctx.facts,
                    &module,
                    &member,
                    &mut Vec::new(),
                );
            }
        }
    }
    let Some(binding) = binding_named(ctx, name) else {
        return Vec::new();
    };
    if is_fragment_binding(ctx.ts, ctx.tags, binding) {
        return ctx
            .catalog
            .local_all(ctx.file)
            .into_iter()
            .filter(|fragment| {
                ctx.ts.exported_bindings.iter().any(|export| {
                    export.specifier.is_none()
                        && export.local == binding.name
                        && export.exported == fragment.exported
                })
            })
            .collect();
    }
    let value = binding.value.clone();
    use_expr(ctx, &value, depth + 1)
}

pub(super) fn member_of(
    ctx: &FileCtx<'_>,
    base: &ValueExpression,
    member: &str,
    depth: u8,
) -> Vec<Fragment> {
    let Some(module) = expr_module(ctx, base, depth) else {
        return Vec::new();
    };
    visible(
        ctx.catalog,
        ctx.graph,
        ctx.facts,
        &module,
        member,
        &mut Vec::new(),
    )
}
