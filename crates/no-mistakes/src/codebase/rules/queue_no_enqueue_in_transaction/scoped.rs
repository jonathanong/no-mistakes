use super::analyze::{
    any_candidate_matches, exact_target, is_ambiguous, site_key, CallKey, Context,
};
use super::reexports::{self, ResolvedApi};
use super::{scoped_finding, unanalyzable};
use crate::codebase::dependencies::extract::{
    CallableId, InvocationKind, ValueCall, ValueFlowFacts,
};
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::fx::{FxHashMap, FxHashSet};

pub(super) struct Lifetime<'a> {
    owner: Option<CallableId>,
    start: u32,
    end: u32,
    method_calls: &'a FxHashSet<(u32, u32)>,
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

pub(super) fn check(
    ctx: &Context<'_>,
    sites: &[ResolvedCallSite],
    flow: &ValueFlowFacts,
    lifetimes: &[Lifetime<'_>],
    calls_by_key: &FxHashMap<CallKey<'_>, Vec<&ValueCall>>,
    diagnostics: &mut AnalysisDiagnostics,
) {
    let owned_methods: FxHashSet<(u32, u32)> = lifetimes
        .iter()
        .flat_map(|lifetime| lifetime.method_calls.iter().copied())
        .collect();
    let mut candidates = FxHashMap::default();
    for lifetime in lifetimes {
        let start = sites.partition_point(|site| site.offset < lifetime.start);
        let end = sites.partition_point(|site| site.offset < lifetime.end);
        for site in &sites[start..end] {
            if site.invocation != InvocationKind::Call {
                continue;
            }
            let key = site_key(site);
            let call = calls_by_key
                .get(&key)
                .and_then(|calls| calls.iter().find(|call| call.context == site.caller_id));
            if call.is_some_and(|call| call.bound) {
                continue;
            }
            let owned_method =
                call.is_some_and(|call| owned_methods.contains(&(call.offset, call.end)));
            let same_execution =
                site.caller_id == lifetime.owner && !flow.deferred_calls.contains(&site.offset);
            if let Some(api) = exact_target(site) {
                match reexports::follow_export(
                    ctx,
                    &site.file,
                    api.0,
                    api.1,
                    &|module, export| ctx.targets.publishers.contains(&(module, export)),
                ) {
                    ResolvedApi::Target(module, export) if same_execution => {
                        candidates.insert(key, (site, Some((module, export))));
                    }
                    ResolvedApi::Target(_, _) | ResolvedApi::Unanalyzable => {
                        candidates.entry(key).or_insert((site, None));
                    }
                    ResolvedApi::Other => {}
                }
            } else if (site.target == ResolvedCallTarget::Unknown && !owned_method)
                || (is_ambiguous(site)
                    && (site.has_unknown_target()
                        || any_candidate_matches(site, |api| {
                            matches!(
                                reexports::follow_export(
                                    ctx,
                                    &site.file,
                                    api.0,
                                    api.1,
                                    &|module, export| {
                                        ctx.targets.publishers.contains(&(module, export))
                                    },
                                ),
                                ResolvedApi::Target(..) | ResolvedApi::Unanalyzable
                            )
                        })))
            {
                candidates.entry(key).or_insert((site, None));
            }
        }
    }
    for (site, api) in candidates.into_values() {
        if let Some((module, export)) = api {
            diagnostics
                .findings
                .push(scoped_finding(ctx, site, (module.as_str(), export.as_str())));
        } else {
            diagnostics.unanalyzable(ctx.policy, unanalyzable(ctx, site,
                "call target or execution timing is unresolved inside the transaction handle scope; call a statically known publisher after commit"));
        }
    }
}
