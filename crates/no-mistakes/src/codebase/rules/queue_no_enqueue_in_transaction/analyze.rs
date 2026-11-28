use super::builtins;
use super::callbacks::CallbackWalk;
use super::options::Targets;
use super::receivers::{invalidated_locals, structural_params, written_members, FileEnv};
use super::reexports;
pub(super) use super::targets::{
    any_candidate_matches, exact_target, is_ambiguous, site_key, CallKey,
};
use super::transactions::collect_boundaries;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    CallableId, ImportedBinding, InvocationKind, ValueCall, ValueExpression, ValueType,
};
use crate::codebase::dependencies::graph::{DepGraph, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::{schema::RuleDef, Unanalyzable};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::Path;

pub(super) struct Context<'a> {
    pub(super) root: &'a Path,
    pub(super) application: &'a RuleDef,
    pub(super) policy: Unanalyzable,
    pub(super) targets: &'a Targets<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) facts: &'a CheckFactMap,
}

pub(super) fn check_file(ctx: &Context<'_>, file: &Path, diagnostics: &mut AnalysisDiagnostics) {
    let Some(file_facts) = ctx
        .facts
        .get_ts_facts(file)
        .filter(|f| f.parse_error.is_none() && f.operational_error.is_none())
    else {
        return;
    };
    let sites = ctx.graph.call_sites_in_file(file);
    let flow = &file_facts.value_flow;
    let imports: FxHashMap<&str, &ImportedBinding> = file_facts
        .imported_bindings
        .iter()
        .chain(flow.commonjs_imported_bindings.iter())
        .filter(|b| !b.is_type_only)
        .map(|b| (b.local.as_str(), b))
        .collect();
    let mut calls_by_key: FxHashMap<CallKey<'_>, Vec<&ValueCall>> = FxHashMap::default();
    let mut calls_by_span: FxHashMap<(u32, u32), &ValueCall> = FxHashMap::default();
    for call in &flow.calls {
        calls_by_key
            .entry((call.offset, call.source_callee.as_str()))
            .or_default()
            .push(call);
        calls_by_span.insert((call.offset, call.end), call);
    }
    let mut sites_by_key: FxHashMap<CallKey<'_>, Vec<&ResolvedCallSite>> = FxHashMap::default();
    let mut sites_by_caller: FxHashMap<CallableId, Vec<&ResolvedCallSite>> = FxHashMap::default();
    let mut construct_offsets = FxHashSet::default();
    for site in sites {
        sites_by_key.entry(site_key(site)).or_default().push(site);
        if site.invocation == InvocationKind::Construct {
            construct_offsets.insert(site.offset);
        } else if site.invocation == InvocationKind::Call
            && !flow.deferred_calls.contains(&site.offset)
        {
            if let Some(caller) = site.caller_id {
                sites_by_caller.entry(caller).or_default().push(site);
            }
        }
    }
    let builtin_construct_offsets = flow
        .calls
        .iter()
        .filter(|c| {
            construct_offsets.contains(&c.offset)
                && matches!(&c.callee,
            ValueExpression::Named { name, scope: None, .. }
                if !imports.contains_key(name.as_str()) && builtins::is_builtin_constructor(name))
        })
        .map(|c| c.offset)
        .collect();
    let boundaries =
        collect_boundaries(ctx, file, sites, &calls_by_key, &imports, flow, diagnostics);
    let tx_callbacks = &boundaries.callbacks;
    let scoped_handle_calls = boundaries
        .handles
        .iter()
        .flat_map(|handle| handle.method_calls.iter().copied())
        .collect();
    let env = FileEnv {
        ctx,
        file,
        file_facts,
        imports,
        calls_by_span,
        sites_by_key,
        builtin_construct_offsets,
        params: flow
            .parameters
            .iter()
            .filter(|p| p.invalidated_at.is_none())
            .map(|p| (p.scope, p.name.as_str()))
            .collect(),
        callback_params: flow
            .parameters
            .iter()
            .filter(|p| tx_callbacks.contains(&p.owner) && p.invalidated_at.is_none())
            .map(|p| (p.owner, p.scope, p.name.as_str()))
            .collect(),
        publisher_typed_params: flow
            .parameters
            .iter()
            .filter(|p| {
                matches!(&p.type_reference, Some(ValueType::Reference(name)) if {
                    let root = name.split_once('.').map_or(name.as_str(), |(root, _)| root);
                    file_facts.imported_bindings.iter().any(|b| {
                        b.local == root
                            && reexports::binding_reaches_module(
                                ctx,
                                file,
                                b,
                                &ctx.targets.publisher_modules,
                            )
                    })
                })
            })
            .map(|p| (p.scope, p.name.as_str()))
            .collect(),
        local_bindings: flow
            .bindings
            .iter()
            .filter(|b| b.immutable && b.invalidated_at.is_none())
            .map(|b| ((b.scope, b.name.as_str()), &b.value))
            .collect(),
        iteration_bindings: flow
            .iteration_bindings
            .iter()
            .filter(|b| b.immutable && b.invalidated_at.is_none())
            .map(|b| ((b.scope, b.name.as_str()), &b.value))
            .collect(),
        local_callables: file_facts
            .callable_bindings
            .iter()
            .map(|(scope, name, id)| ((*scope, name.as_str()), *id))
            .collect(),
        structural_params: structural_params(&flow.parameters),
        invalidated_locals: invalidated_locals(flow, &file_facts.callable_binding_declared_at),
        written_members: written_members(flow),
        scoped_handle_calls,
    };
    super::scoped::check(
        &env,
        sites,
        flow,
        &boundaries.handles,
        &calls_by_key,
        &sites_by_caller,
        diagnostics,
    );
    let mut walk = CallbackWalk {
        visited: tx_callbacks.iter().copied().map(|cb| (cb, cb)).collect(),
        worklist: tx_callbacks.iter().map(|&cb| (cb, cb)).collect(),
        reported: FxHashSet::default(),
        diagnostics,
    };
    let mut cursors: FxHashMap<(CallableId, CallableId, u32, &str), usize> = FxHashMap::default();
    super::analyze_callback::drain_callbacks(
        &env,
        &calls_by_key,
        &sites_by_caller,
        &mut walk,
        &mut cursors,
    );
}
