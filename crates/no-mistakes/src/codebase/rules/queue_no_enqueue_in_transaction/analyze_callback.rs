use super::analyze::{any_candidate_matches, exact_target, is_ambiguous};
use super::builtins;
use super::callbacks::CallbackWalk;
use super::reexports::{self, ResolvedApi};
use super::receivers::{classify_builtin_site, is_callback_param_handle, FileEnv};
use crate::codebase::dependencies::extract::{CallableId, ValueCall};
use crate::codebase::dependencies::graph::{
    KnownCallTargetRef, ResolvedCallSite, ResolvedCallTarget,
};

pub(super) fn check_callback_site(
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
                &|module, export| ctx.targets.publishers.contains(&(module, export)),
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
                reexports::follow_export(ctx, env.file, api.0, api.1, &|module, export| {
                    ctx.targets.publishers.contains(&(module, export))
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
        if let Some((call, builtin)) = call
            .and_then(|c| classify_builtin_site(env, site, c, tx_owner, 0).map(|builtin| (c, builtin)))
        {
            walk.inspect_builtin(env, site, call, builtin, tx_owner);
        } else {
            walk.report_unanalyzable(env, site,
                "unresolved call target inside transaction callback cannot be proved not to publish; use a statically resolvable call or move publishes after commit");
        }
    }
}

fn any_global_callback(site: &ResolvedCallSite) -> bool {
    site.targets().any(|candidate| {
        matches!(
            candidate.target,
            KnownCallTargetRef::Global { name } if builtins::global_target_callbacks(name).is_some()
        )
    }) || site.alternatives.iter().any(any_global_callback)
}
