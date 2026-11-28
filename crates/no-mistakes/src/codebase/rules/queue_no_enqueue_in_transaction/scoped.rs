use super::analyze::{any_candidate_matches, exact_target, is_ambiguous, site_key, CallKey};
use super::analyze_callback::drain_callbacks;
use super::builtins;
use super::callbacks::CallbackWalk;
use super::receivers::{classify_builtin_site, FileEnv};
use super::reexports::{self, ResolvedApi};
use super::{scoped_finding, unanalyzable};
use crate::codebase::dependencies::extract::{
    CallableId, InvocationKind, ValueCall, ValueExpression, ValueFlowFacts,
};
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::fx::{FxHashMap, FxHashSet};

pub(super) struct Lifetime<'a> {
    owner: Option<CallableId>,
    start: u32,
    end: u32,
    pub(super) method_calls: &'a FxHashSet<(u32, u32)>,
}

pub(super) fn lifetime<'a>(
    flow: &'a ValueFlowFacts,
    direct: Option<&ValueCall>,
) -> Result<Lifetime<'a>, &'static str> {
    let Some(handle) =
        direct.and_then(|call| flow.scoped_handles.starts.get(&(call.offset, call.end)))
    else {
        return Err("transaction handle initializer is not a direct begin call assigned to a single local binding; bind the imported call directly");
    };
    if handle.uncertain {
        return Err("transaction handle escapes or may close outside the straight-line scope; keep the handle local and await commit or rollback directly");
    }
    let end = match handle.close {
        Some((_, end)) => end,
        None if handle.using => handle.scope_end,
        None => return Err("transaction handle has no proven commit or rollback before this scope ends; await commit or rollback directly or use await using"),
    };
    Ok(Lifetime {
        owner: handle.owner,
        start: handle.after_begin,
        end,
        method_calls: &handle.method_calls,
    })
}

pub(super) fn is_owned_handle_call(env: &FileEnv<'_>, call: &ValueCall, depth: u8) -> bool {
    if env.scoped_handle_calls.contains(&(call.offset, call.end)) {
        return true;
    }
    if depth > 6 {
        return false;
    }
    let ValueExpression::Member(receiver, _) = &call.callee else {
        return false;
    };
    let inner = match receiver.as_ref() {
        ValueExpression::Call(start, end) => Some((*start, *end)),
        ValueExpression::Awaited(expr) => match expr.as_ref() {
            ValueExpression::Call(start, end) => Some((*start, *end)),
            _ => None,
        },
        _ => None,
    };
    inner
        .and_then(|span| env.calls_by_span.get(&span).copied())
        .is_some_and(|call| is_owned_handle_call(env, call, depth + 1))
}

pub(super) fn check(
    env: &FileEnv<'_>,
    sites: &[ResolvedCallSite],
    flow: &ValueFlowFacts,
    lifetimes: &[Lifetime<'_>],
    calls_by_key: &FxHashMap<CallKey<'_>, Vec<&ValueCall>>,
    sites_by_caller: &FxHashMap<CallableId, Vec<&ResolvedCallSite>>,
    diagnostics: &mut AnalysisDiagnostics,
) {
    if lifetimes.is_empty() {
        return;
    }
    let mut candidates = FxHashMap::default();
    let mut walk = CallbackWalk {
        visited: FxHashSet::default(),
        worklist: Vec::new(),
        reported: FxHashSet::default(),
        diagnostics,
    };
    let mut cursors: FxHashMap<(CallableId, CallableId, u32, &str), usize> = FxHashMap::default();
    for lifetime in lifetimes {
        let tx_owner = lifetime.owner.unwrap_or(CallableId(0));
        let start = sites.partition_point(|site| site.offset < lifetime.start);
        let end = sites.partition_point(|site| site.offset < lifetime.end);
        for site in &sites[start..end] {
            if site.invocation != InvocationKind::Call
                || site
                    .caller_id
                    .is_some_and(|caller| walk.visited.contains(&(caller, tx_owner)))
            {
                continue;
            }
            let key = site_key(site);
            let call = calls_by_key
                .get(&key)
                .and_then(|calls| calls.iter().find(|call| call.context == site.caller_id))
                .copied();
            if call.is_some_and(|call| call.bound) {
                continue;
            }
            let owned_method = call.is_some_and(|call| is_owned_handle_call(env, call, 0));
            let same_execution =
                site.caller_id == lifetime.owner && !flow.deferred_calls.contains(&site.offset);
            if let Some(api) = exact_target(site) {
                match reexports::follow_export(
                    env.ctx,
                    &site.file,
                    api.0,
                    api.1,
                    &|module, export| env.ctx.targets.publishers.contains(&(module, export)),
                ) {
                    ResolvedApi::Target(module, export) if same_execution => {
                        candidates.insert(key, (site, Some((module, export))));
                    }
                    ResolvedApi::Target(_, _) | ResolvedApi::Unanalyzable => {
                        candidates.entry(key).or_insert((site, None));
                    }
                    ResolvedApi::Other => {}
                }
            } else if let ResolvedCallTarget::Global { name } = &site.target {
                if same_execution {
                    if let (Some(builtin), Some(call)) =
                        (builtins::global_target_callbacks(name), call)
                    {
                        walk.inspect_builtin(env, site, call, builtin, tx_owner);
                        drain_callbacks(
                            env,
                            calls_by_key,
                            sites_by_caller,
                            &mut walk,
                            &mut cursors,
                        );
                    }
                }
            } else if site.target == ResolvedCallTarget::Unknown && !owned_method {
                if same_execution {
                    if let Some((call, builtin)) = call.and_then(|c| {
                        classify_builtin_site(env, site, c, tx_owner, 0).map(|b| (c, b))
                    }) {
                        walk.inspect_builtin(env, site, call, builtin, tx_owner);
                        drain_callbacks(
                            env,
                            calls_by_key,
                            sites_by_caller,
                            &mut walk,
                            &mut cursors,
                        );
                        continue;
                    }
                }
                candidates.entry(key).or_insert((site, None));
            } else if is_ambiguous(site)
                && (site.has_unknown_target()
                    || any_candidate_matches(site, |api| {
                        matches!(
                            reexports::follow_export(
                                env.ctx,
                                &site.file,
                                api.0,
                                api.1,
                                &|module, export| {
                                    env.ctx.targets.publishers.contains(&(module, export))
                                },
                            ),
                            ResolvedApi::Target(..) | ResolvedApi::Unanalyzable
                        )
                    }))
            {
                candidates.entry(key).or_insert((site, None));
            }
        }
    }
    for (site, api) in candidates.into_values() {
        if let Some((module, export)) = api {
            walk.diagnostics.findings.push(scoped_finding(
                env.ctx,
                site,
                (module.as_str(), export.as_str()),
            ));
        } else {
            walk.diagnostics.unanalyzable(env.ctx.policy, unanalyzable(env.ctx, site,
                "call target or execution timing is unresolved inside the transaction handle scope; call a statically known publisher after commit"));
        }
    }
}
