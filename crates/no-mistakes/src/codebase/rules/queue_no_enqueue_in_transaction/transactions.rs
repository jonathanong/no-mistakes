use super::analyze::{any_candidate_matches, exact_target, is_ambiguous, site_key, CallKey, Context};
use super::options::Api;
use super::reexports::{self, ResolvedApi};
use super::unanalyzable;
use crate::codebase::dependencies::extract::{
    CallableId, ImportedBinding, InvocationKind, ValueCall, ValueExpression, ValueFlowFacts,
};
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::Path;

const UNRESOLVED_TX_MSG: &str = "unresolved call target may invoke a configured transaction (dynamic member, factory return, or wildcard re-export); import and call the transaction directly";

pub(super) fn collect_tx_callbacks(
    ctx: &Context<'_>,
    file: &Path,
    sites: &[ResolvedCallSite],
    calls_by_key: &FxHashMap<CallKey<'_>, Vec<&ValueCall>>,
    imports: &FxHashMap<&str, &ImportedBinding>,
    flow: &ValueFlowFacts,
    diagnostics: &mut AnalysisDiagnostics,
) -> FxHashSet<CallableId> {
    let mut callbacks = FxHashSet::default();
    let mut deferred_unanalyzable: Vec<&ResolvedCallSite> = Vec::new();
    let mut cursors: FxHashMap<CallKey<'_>, usize> = FxHashMap::default();
    for site in sites.iter().filter(|site| site.invocation == InvocationKind::Call) {
        let key = site_key(site);
        let cursor = cursors.entry(key).or_insert(0);
        let call = calls_by_key.get(&key).and_then(|list| list.get(*cursor).copied());
        *cursor += 1;
        if let Some(api) = exact_target(site) {
            match resolve_transaction(ctx, file, api) {
                TxResolution::Index(index) => {
                    let non_tagged = call.filter(|c| !flow.tagged_templates.contains_key(&(c.offset, c.end)));
                    match callback_for_transaction(site, index, non_tagged, imports, &flow.generator_callables) {
                        Ok(cb) => { callbacks.insert(cb); }
                        Err(reason) => diagnostics.unanalyzable(ctx.policy, unanalyzable(ctx, site, reason)),
                    }
                }
                TxResolution::Unanalyzable => deferred_unanalyzable.push(site),
                TxResolution::Other => {}
            }
        } else if is_ambiguous(site)
            && any_candidate_matches(site, |api| !matches!(resolve_transaction(ctx, file, api), TxResolution::Other))
        {
            diagnostics.unanalyzable(ctx.policy, unanalyzable(ctx, site,
                "ambiguous call target may invoke a configured transaction; call the imported transaction directly"));
        } else if site.target == ResolvedCallTarget::Unknown
            && unresolved_transaction_site(ctx, file, site, call, imports)
        {
            diagnostics.unanalyzable(ctx.policy, unanalyzable(ctx, site, UNRESOLVED_TX_MSG));
        }
    }
    for site in deferred_unanalyzable {
        if !site.caller_id.is_some_and(|caller| callbacks.contains(&caller)) {
            diagnostics.unanalyzable(ctx.policy, unanalyzable(ctx, site, UNRESOLVED_TX_MSG));
        }
    }
    callbacks
}

enum TxResolution {
    Index(usize),
    Other,
    Unanalyzable,
}

fn resolve_transaction(ctx: &Context<'_>, file: &Path, api: Api<'_>) -> TxResolution {
    match reexports::follow_export(ctx, file, api.0, api.1, &|m, e| {
        ctx.targets.transactions.contains_key(&(m, e))
    }) {
        ResolvedApi::Target(m, e) => TxResolution::Index(
            ctx.targets.transactions[&(m.as_str(), e.as_str())],
        ),
        ResolvedApi::Other => TxResolution::Other,
        ResolvedApi::Unanalyzable => TxResolution::Unanalyzable,
    }
}

fn callback_for_transaction(
    site: &ResolvedCallSite,
    index: usize,
    call: Option<&ValueCall>,
    imports: &FxHashMap<&str, &ImportedBinding>,
    generators: &FxHashSet<CallableId>,
) -> Result<CallableId, &'static str> {
    let Some(call) = call else {
        return Err("transaction call uses a tagged template, so callback argument position cannot be proved; call the transaction directly with an inline callback");
    };
    if call.bound
        || !site.returned_from.is_empty()
        || !site.returned_member_from.is_empty()
        || !root_ident(&call.callee).is_some_and(|name| imports.contains_key(name))
    {
        return Err("transaction call is not a direct import binding (for example bound, factory-returned, or aliased), so argument positions cannot be proved; call the imported transaction directly");
    }
    if call.spread_at.is_some_and(|spread| spread <= index) {
        return Err("transaction call has a spread argument at or before the callback slot; pass arguments positionally with an inline callback");
    }
    let Some(ValueExpression::Callable(callback)) = call.arguments.get(index) else {
        return Err("transaction callback argument is not an inline function expression; pass an inline arrow or function expression");
    };
    if generators.contains(callback) {
        return Err("transaction callback is a generator function, whose execution timing cannot be proved; use a regular or async inline callback");
    }
    Ok(*callback)
}

fn unresolved_transaction_site(
    ctx: &Context<'_>,
    file: &Path,
    site: &ResolvedCallSite,
    call: Option<&ValueCall>,
    imports: &FxHashMap<&str, &ImportedBinding>,
) -> bool {
    let (mods, txs) = (&ctx.targets.transaction_modules, &ctx.targets.transactions);
    for (m, e) in &site.returned_from {
        if reexports::reaches_module(ctx, file, m, None, mods)
            && !txs.contains_key(&(m.as_str(), e.as_str()))
        {
            return true;
        }
    }
    for (m, e, _) in &site.returned_member_from {
        if reexports::reaches_module(ctx, file, m, None, mods)
            && !txs.contains_key(&(m.as_str(), e.as_str()))
        {
            return true;
        }
    }
    let Some(binding) = call.and_then(|c| root_ident(&c.callee)).and_then(|name| imports.get(name)) else {
        return false;
    };
    reexports::binding_reaches_module(ctx, file, binding, mods)
}

fn root_ident(mut expr: &ValueExpression) -> Option<&str> {
    while let ValueExpression::Member(receiver, _) = expr {
        expr = receiver;
    }
    match expr {
        ValueExpression::Named { name, scope: None | Some(0), .. } => Some(name.as_str()),
        _ => None,
    }
}
