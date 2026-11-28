//! Rule-local re-export and reachability resolution (unification with graph export resolution tracked in #4791).
use super::analyze::Context;
use crate::codebase::dependencies::extract::{ImportedBinding, ImportedBindingKind};
use crate::codebase::dependencies::graph::{ImportTarget, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[path = "reexports_local.rs"]
mod local;
#[path = "reexports_reach.rs"]
mod reach;

pub(super) use reach::{binding_reaches_module, reaches_module};

pub(super) enum ResolvedApi {
    Target(String, String),
    Other,
    Unanalyzable,
}

pub(super) fn resolve_binding_api(
    ctx: &Context<'_>,
    from: &Path,
    binding: &ImportedBinding,
    suffix: Option<&str>,
    is_target: impl Fn(&str, &str) -> bool,
) -> Option<ResolvedApi> {
    let export = project_imported(binding, suffix)?;
    Some(follow_export(ctx, from, &binding.specifier, &export, &is_target))
}

pub(super) fn follow_export(
    ctx: &Context<'_>,
    from: &Path,
    specifier: &str,
    export: &str,
    is_target: &impl Fn(&str, &str) -> bool,
) -> ResolvedApi {
    walk(ctx, from, specifier, export, is_target, &mut FxHashMap::default())
}

fn walk(
    ctx: &Context<'_>,
    from: &Path,
    specifier: &str,
    export: &str,
    is_target: &impl Fn(&str, &str) -> bool,
    visited: &mut FxHashMap<(PathBuf, String), usize>,
) -> ResolvedApi {
    if is_target(specifier, export) {
        return ResolvedApi::Target(specifier.to_owned(), export.to_owned());
    }
    let Some(ImportTarget::File(target)) =
        ctx.graph.import_targets.get(from).and_then(|m| m.get(specifier))
    else {
        return if specifier.starts_with(['.', '/']) {
            ResolvedApi::Unanalyzable
        } else {
            ResolvedApi::Other
        };
    };
    let head = export.split_once('.').map_or(export, |(head, _)| head);
    let key = (target.clone(), head.to_owned());
    if visited.get(&key).is_some_and(|&prev| export.len() >= prev) {
        return ResolvedApi::Unanalyzable;
    }
    visited.insert(key, export.len());
    let Some(tf) = ctx
        .facts
        .get_ts_facts(target)
        .filter(|f| f.parse_error.is_none() && f.operational_error.is_none())
    else {
        return ResolvedApi::Unanalyzable;
    };
    let (head, rest) = split_export(export);
    let Some(eb) = tf.exported_bindings.iter().find(|eb| eb.exported == head) else {
        return ResolvedApi::Unanalyzable;
    };
    if let Some(next_spec) = &eb.specifier {
        let Some(next_export) = project_export(&eb.local, rest) else {
            return ResolvedApi::Unanalyzable;
        };
        return walk(ctx, target, next_spec, &next_export, is_target, visited);
    }
    let Some(imp) = find_import(tf, &eb.local) else {
        return local::classify_local_export(ctx, target, tf, &eb.local, rest, is_target, visited);
    };
    let Some(next_export) = project_imported(imp, rest) else {
        return ResolvedApi::Unanalyzable;
    };
    walk(ctx, target, &imp.specifier, &next_export, is_target, visited)
}

fn find_import<'a>(tf: &'a TsFileFacts, local: &str) -> Option<&'a ImportedBinding> {
    tf.imported_bindings
        .iter()
        .chain(tf.value_flow.commonjs_imported_bindings.iter())
        .find(|b| !b.is_type_only && b.local == local)
}

fn split_export(export: &str) -> (&str, Option<&str>) {
    export.split_once('.').map_or((export, None), |(h, r)| (h, Some(r)))
}

fn project_export(local: &str, rest: Option<&str>) -> Option<String> {
    match (local, rest) {
        ("*", Some(r)) => Some(r.to_owned()),
        ("*", None) => None,
        (name, Some(r)) => Some(format!("{name}.{r}")),
        (name, None) => Some(name.to_owned()),
    }
}

fn project_imported(imp: &ImportedBinding, rest: Option<&str>) -> Option<String> {
    match (imp.kind, rest) {
        (ImportedBindingKind::Namespace, Some(r)) => Some(r.to_owned()),
        (ImportedBindingKind::Namespace, None) => None,
        (_, Some(r)) => Some(format!("{}.{r}", imp.imported)),
        (_, None) => Some(imp.imported.clone()),
    }
}
