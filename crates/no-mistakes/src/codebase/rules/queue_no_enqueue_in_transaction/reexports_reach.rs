use super::super::analyze::Context;
use super::{local, ResolvedApi};
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::dependencies::graph::{ImportTarget, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

pub(in super::super) fn binding_reaches_module(
    ctx: &Context<'_>,
    from: &Path,
    binding: &ImportedBinding,
    modules: &FxHashSet<&str>,
) -> bool {
    let export =
        (binding.kind != ImportedBindingKind::Namespace).then_some(binding.imported.as_str());
    reaches_module(ctx, from, &binding.specifier, export, modules)
}

pub(in super::super) fn reaches_module(
    ctx: &Context<'_>,
    from: &Path,
    specifier: &str,
    export: Option<&str>,
    modules: &FxHashSet<&str>,
) -> bool {
    walk(
        ctx,
        from,
        specifier,
        export,
        modules,
        &mut ReachState {
            stack: FxHashSet::default(),
            memo: FxHashMap::default(),
        },
    )
}

struct ReachState {
    stack: FxHashSet<(PathBuf, Option<String>)>,
    memo: FxHashMap<(PathBuf, Option<String>), bool>,
}

fn walk(
    ctx: &Context<'_>,
    from: &Path,
    spec: &str,
    export: Option<&str>,
    modules: &FxHashSet<&str>,
    state: &mut ReachState,
) -> bool {
    if modules.contains(spec) {
        return true;
    }
    let Some(ImportTarget::File(target)) =
        ctx.graph.import_targets.get(from).and_then(|m| m.get(spec))
    else {
        return spec.starts_with(['.', '/']);
    };
    let key = (target.clone(), export.map(str::to_owned));
    if let Some(done) = state.memo.get(&key) {
        return *done;
    }
    if !state.stack.insert(key.clone()) {
        return true;
    }
    let reached = walk_target(ctx, target, export, modules, state);
    state.stack.remove(&key);
    state.memo.insert(key, reached);
    reached
}

fn walk_target(
    ctx: &Context<'_>,
    target: &Path,
    export: Option<&str>,
    modules: &FxHashSet<&str>,
    state: &mut ReachState,
) -> bool {
    let Some(tf) = ctx
        .facts
        .get_ts_facts(target)
        .filter(|f| f.parse_error.is_none() && f.operational_error.is_none())
    else {
        return true;
    };
    let mut exports = tf
        .exported_bindings
        .iter()
        .chain(tf.value_flow.type_exports.iter());
    if let Some(name) = export {
        let mut matched = false;
        for eb in exports.filter(|eb| eb.exported == name) {
            matched = true;
            if export_binding_reaches(ctx, target, tf, eb, modules, state) {
                return true;
            }
        }
        return !matched
            && tf
                .star_reexport_specifiers
                .iter()
                .any(|s| walk(ctx, target, s, Some(name), modules, state));
    }
    tf.star_reexport_specifiers
        .iter()
        .any(|s| walk(ctx, target, s, None, modules, state))
        || exports.any(|eb| export_binding_reaches(ctx, target, tf, eb, modules, state))
}

fn export_binding_reaches(
    ctx: &Context<'_>,
    target: &Path,
    tf: &TsFileFacts,
    eb: &ExportedBinding,
    modules: &FxHashSet<&str>,
    state: &mut ReachState,
) -> bool {
    if let Some(next_spec) = &eb.specifier {
        let next_export = (eb.local != "*").then_some(eb.local.as_str());
        return walk(ctx, target, next_spec, next_export, modules, state);
    }
    if let Some(imp) = tf
        .imported_bindings
        .iter()
        .chain(tf.value_flow.commonjs_imported_bindings.iter())
        .find(|b| b.local == eb.local)
    {
        let next_export =
            (imp.kind != ImportedBindingKind::Namespace).then_some(imp.imported.as_str());
        return walk(ctx, target, &imp.specifier, next_export, modules, state);
    }
    !matches!(
        local::classify_local_export(
            ctx,
            target,
            tf,
            &eb.local,
            None,
            &|spec, _| modules.contains(spec),
            &mut FxHashMap::default(),
        ),
        ResolvedApi::Other
    )
}
