use super::builtins;
use super::callbacks::CallbackWalk;
use super::options::{Api, Targets};
use super::receivers::{
    classify_builtin_site, invalidated_locals, is_callback_param_handle, structural_params,
    written_members, FileEnv,
};
use super::reexports::{self, ResolvedApi};
use super::transactions::collect_tx_callbacks;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    CallableId, ImportedBinding, InvocationKind, ValueCall, ValueExpression, ValueType,
};
use crate::codebase::dependencies::graph::{
    DepGraph, KnownCallTargetRef, ResolvedCallSite, ResolvedCallTarget, TsFactLookup,
};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::{schema::RuleDef, Unanalyzable};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::Path;

pub(super) type CallKey<'a> = (u32, &'a str);

pub(super) struct Context<'a> {
    pub(super) root: &'a Path,
    pub(super) application: &'a RuleDef,
    pub(super) policy: Unanalyzable,
    pub(super) targets: &'a Targets<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) facts: &'a CheckFactMap,
}

pub(super) fn check_file(ctx: &Context<'_>, file: &Path, diagnostics: &mut AnalysisDiagnostics) {
    let Some(file_facts) = ctx.facts.get_ts_facts(file)
        .filter(|f| f.parse_error.is_none() && f.operational_error.is_none()) else { return; };
    let sites = ctx.graph.call_sites_in_file(file);
    let flow = &file_facts.value_flow;
    let imports: FxHashMap<&str, &ImportedBinding> = file_facts.imported_bindings.iter()
        .chain(flow.commonjs_imported_bindings.iter())
        .filter(|b| !b.is_type_only).map(|b| (b.local.as_str(), b)).collect();
    let mut calls_by_key: FxHashMap<CallKey<'_>, Vec<&ValueCall>> = FxHashMap::default();
    let mut calls_by_span: FxHashMap<(u32, u32), &ValueCall> = FxHashMap::default();
    for call in &flow.calls {
        calls_by_key.entry((call.offset, call.source_callee.as_str())).or_default().push(call);
        calls_by_span.insert((call.offset, call.end), call);
    }
    let mut sites_by_key: FxHashMap<CallKey<'_>, Vec<&ResolvedCallSite>> = FxHashMap::default();
    let mut sites_by_caller: FxHashMap<CallableId, Vec<&ResolvedCallSite>> = FxHashMap::default();
    let mut construct_offsets = FxHashSet::default();
    for site in sites {
        sites_by_key.entry(site_key(site)).or_default().push(site);
        if site.invocation == InvocationKind::Construct {
            construct_offsets.insert(site.offset);
        } else if site.invocation == InvocationKind::Call && !flow.deferred_calls.contains(&site.offset) {
            if let Some(caller) = site.caller_id {
                sites_by_caller.entry(caller).or_default().push(site);
            }
        }
    }
    let builtin_construct_offsets = flow.calls.iter()
        .filter(|c| construct_offsets.contains(&c.offset) && matches!(&c.callee,
            ValueExpression::Named { name, scope: None, .. }
                if !imports.contains_key(name.as_str()) && builtins::is_builtin_constructor(name)))
        .map(|c| c.offset).collect();
    let tx_callbacks = collect_tx_callbacks(ctx, file, sites, &calls_by_key, &imports, flow, diagnostics);
    let env = FileEnv {
        ctx, file, file_facts, imports, calls_by_span, sites_by_key, builtin_construct_offsets,
        params: flow.parameters.iter()
            .filter(|p| p.invalidated_at.is_none())
            .map(|p| (p.scope, p.name.as_str())).collect(),
        callback_params: flow.parameters.iter()
            .filter(|p| tx_callbacks.contains(&p.owner) && p.invalidated_at.is_none())
            .map(|p| (p.owner, p.scope, p.name.as_str())).collect(),
        publisher_typed_params: flow.parameters.iter()
            .filter(|p| matches!(&p.type_reference, Some(ValueType::Reference(name)) if {
                let root = name.split_once('.').map_or(name.as_str(), |(root, _)| root);
                file_facts.imported_bindings.iter().any(|b| {
                    b.local == root
                        && reexports::binding_reaches_module(ctx, file, b, &ctx.targets.publisher_modules)
                })
            }))
            .map(|p| (p.scope, p.name.as_str())).collect(),
        local_bindings: flow.bindings.iter()
            .filter(|b| b.immutable && b.invalidated_at.is_none())
            .map(|b| ((b.scope, b.name.as_str()), &b.value)).collect(),
        iteration_bindings: flow.iteration_bindings.iter()
            .filter(|b| b.immutable && b.invalidated_at.is_none())
            .map(|b| ((b.scope, b.name.as_str()), &b.value)).collect(),
        local_callables: file_facts.callable_bindings.iter()
            .map(|(scope, name, id)| ((*scope, name.as_str()), *id)).collect(),
        structural_params: structural_params(&flow.parameters),
        invalidated_locals: invalidated_locals(flow, &file_facts.callable_binding_declared_at),
        written_members: written_members(flow),
    };
    let mut walk = CallbackWalk {
        visited: tx_callbacks.iter().copied().map(|cb| (cb, cb)).collect(),
        worklist: tx_callbacks.iter().map(|&cb| (cb, cb)).collect(),
        reported: FxHashSet::default(),
        diagnostics,
    };
    let mut cursors: FxHashMap<(CallableId, CallableId, u32, &str), usize> = FxHashMap::default();
    while let Some((caller, tx_owner)) = walk.worklist.pop() {
        let Some(caller_sites) = sites_by_caller.get(&caller) else { continue; };
        for site in caller_sites {
            let key = site_key(site);
            let cursor = cursors.entry((tx_owner, caller, key.0, key.1)).or_insert(0);
            let call = calls_by_key.get(&key)
                .and_then(|list| list.iter().filter(|c| c.context == Some(caller)).nth(*cursor).copied());
            *cursor += 1;
            check_callback_site(&env, site, call, tx_owner, &mut walk);
        }
    }
}

fn check_callback_site(
    env: &FileEnv<'_>,
    site: &ResolvedCallSite,
    call: Option<&ValueCall>,
    tx_owner: CallableId,
    walk: &mut CallbackWalk<'_>,
) {
    let ctx = env.ctx;
    if let Some(api) = exact_target(site) {
        if !call.is_some_and(|c| c.bound) {
            match reexports::follow_export(
                ctx,
                env.file,
                api.0,
                api.1,
                &|m, e| ctx.targets.publishers.contains(&(m, e)),
            ) {
                ResolvedApi::Target(spec, export) => {
                    walk.report_publish(env, site, (spec.as_str(), export.as_str()));
                }
                ResolvedApi::Unanalyzable => {
                    walk.report_unanalyzable(env, site,
                        "unresolved call target inside transaction callback cannot be proved not to publish; use a statically resolvable call or move publishes after commit");
                }
                ResolvedApi::Other => {}
            }
        }
    } else if is_ambiguous(site)
        && (any_candidate_matches(site, |api| {
            matches!(
                reexports::follow_export(ctx, env.file, api.0, api.1, &|m, e| {
                    ctx.targets.publishers.contains(&(m, e))
                }),
                ResolvedApi::Target(..) | ResolvedApi::Unanalyzable
            )
        }) || site.has_unknown_target() || any_global_callback(site))
    {
        walk.report_unanalyzable(env, site,
            "ambiguous call target inside transaction callback may invoke a configured publisher; call a single known target or move publishes after commit");
    } else if let ResolvedCallTarget::Global { name } = &site.target {
        if let (Some(builtin), Some(call)) = (builtins::global_target_callbacks(name), call) {
            walk.inspect_builtin(env, site, call, builtin, tx_owner);
        }
    } else if site.target == ResolvedCallTarget::Unknown
        && !is_callback_param_handle(call, tx_owner, &env.callback_params)
    {
        if let Some((call, builtin)) = call.and_then(|c| {
            classify_builtin_site(env, site, c, tx_owner, 0).map(|builtin| (c, builtin))
        }) {
            walk.inspect_builtin(env, site, call, builtin, tx_owner);
        } else {
            walk.report_unanalyzable(env, site,
                "unresolved call target inside transaction callback cannot be proved not to publish; use a statically resolvable call or move publishes after commit");
        }
    }
}

pub(super) fn exact_target(site: &ResolvedCallSite) -> Option<Api<'_>> {
    if !site.alternatives.is_empty() { return None; }
    match &site.target {
        ResolvedCallTarget::ModuleExport { specifier, export_path, .. } => Some((specifier, export_path)),
        _ => None,
    }
}

pub(super) fn is_ambiguous(site: &ResolvedCallSite) -> bool {
    !site.alternatives.is_empty() || matches!(site.target, ResolvedCallTarget::Alternatives(_))
}

pub(super) fn any_candidate_matches(site: &ResolvedCallSite, predicate: impl Fn(Api<'_>) -> bool + Copy) -> bool {
    site.targets().any(|candidate| match candidate.target {
        KnownCallTargetRef::ModuleExport { specifier, export_path, .. } => predicate((specifier, export_path)),
        _ => false,
    }) || site.alternatives.iter().any(|alt| any_candidate_matches(alt, predicate))
}

pub(super) fn site_key(site: &ResolvedCallSite) -> CallKey<'_> { (site.offset, site.source_callee.as_str()) }

fn any_global_callback(site: &ResolvedCallSite) -> bool {
    site.targets().any(|candidate| {
        matches!(
            candidate.target,
            KnownCallTargetRef::Global { name } if builtins::global_target_callbacks(name).is_some()
        )
    }) || site.alternatives.iter().any(any_global_callback)
}
