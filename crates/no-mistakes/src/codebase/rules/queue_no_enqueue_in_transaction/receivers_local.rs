use super::{
    classify_builtin_site, is_callback_param_handle, is_local_or_builtin_receiver, FileEnv,
};
use crate::codebase::dependencies::extract::{
    CallableId, ImportedBinding, InvocationKind, ValueExpression,
};
use crate::codebase::dependencies::graph::ResolvedCallTarget;
use crate::codebase::rules::queue_no_enqueue_in_transaction::analyze::exact_target;
use crate::codebase::rules::queue_no_enqueue_in_transaction::builtins;
use crate::codebase::rules::queue_no_enqueue_in_transaction::reexports::{self, ResolvedApi};
use crate::fx::FxHashMap;

pub(super) fn is_clean_scoped_local(
    env: &FileEnv<'_>,
    scope: usize,
    name: &str,
    method: &str,
    call_offset: u32,
    tx_owner: CallableId,
    depth: u8,
) -> bool {
    if env.publisher_typed_params.contains(&(scope, name))
        || (scope == 0 && env.imports.contains_key(name))
        || env.has_member_write(scope, name, method)
    {
        return false;
    }
    if env.params.contains(&(scope, name)) {
        return true;
    }
    if let Some(iter) = env.iteration_bindings.get(&(scope, name)) {
        return is_local_or_builtin_receiver(
            env,
            iter,
            method,
            call_offset,
            tx_owner,
            depth.saturating_add(1),
        );
    }
    if let Some(ctor) = env
        .file_facts
        .value_flow
        .builtin_const_bindings
        .get(&(scope, name.to_owned()))
    {
        return ctor.as_deref().is_none_or(builtins::is_builtin_constructor);
    }
    let Some(bound) = env.local_bindings.get(&(scope, name)) else {
        return false;
    };
    !matches!(bound, ValueExpression::Unknown)
        && is_local_or_builtin_receiver(
            env,
            bound,
            method,
            call_offset,
            tx_owner,
            depth.saturating_add(1),
        )
}

pub(super) fn inner_call_is_clean(
    env: &FileEnv<'_>,
    start: u32,
    end: u32,
    tx_owner: CallableId,
    depth: u8,
) -> bool {
    env.calls_by_span
        .get(&(start, end))
        .copied()
        .is_some_and(|inner| {
            is_callback_param_handle(Some(inner), tx_owner, &env.callback_params)
                || super::super::scoped::is_owned_handle_call(env, inner, 0)
                || env
                    .sites_by_key
                    .get(&(inner.offset, inner.source_callee.as_str()))
                    .is_some_and(|sites| {
                        sites.iter().any(|site| {
                            site.invocation == InvocationKind::Call
                                && match &site.target {
                                    ResolvedCallTarget::RepositoryFunction { .. }
                                    | ResolvedCallTarget::Global { .. } => true,
                                    ResolvedCallTarget::ModuleExport { .. } => exact_target(site)
                                        .is_some_and(|api| {
                                            matches!(
                                                reexports::follow_export(
                                                    env.ctx,
                                                    env.file,
                                                    api.0,
                                                    api.1,
                                                    &|m, e| {
                                                        env.ctx.targets.publishers.contains(&(m, e))
                                                    },
                                                ),
                                                ResolvedApi::Other
                                            ) && !reexports::reaches_module(
                                                env.ctx,
                                                env.file,
                                                api.0,
                                                Some(api.1),
                                                &env.ctx.targets.publisher_modules,
                                            )
                                        }),
                                    ResolvedCallTarget::Unknown => {
                                        classify_builtin_site(env, site, inner, tx_owner, depth)
                                            .is_some()
                                    }
                                    ResolvedCallTarget::Alternatives(_) => false,
                                }
                        })
                    })
        })
}

pub(super) fn expr_is_import(
    mut expr: &ValueExpression,
    imports: &FxHashMap<&str, &ImportedBinding>,
) -> bool {
    while let ValueExpression::Member(inner, _) = expr {
        expr = inner.as_ref();
    }
    matches!(expr, ValueExpression::Named { name, scope: None | Some(0), .. } if imports.contains_key(name.as_str()))
}
