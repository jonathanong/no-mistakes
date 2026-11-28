use super::analyze::{CallKey, Context};
use super::builtins::{self, BuiltinCall};
use super::reexports;
use crate::codebase::dependencies::extract::{
    CallableId, ImportedBinding, ValueCall, ValueExpression,
};
use crate::codebase::dependencies::graph::ResolvedCallSite;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::Path;

#[path = "receivers_local.rs"]
mod local;
#[path = "receivers_sets.rs"]
mod sets;

pub(super) use sets::{invalidated_locals, structural_params, written_members};

pub(super) struct FileEnv<'a> {
    pub(super) ctx: &'a Context<'a>,
    pub(super) file: &'a Path,
    pub(super) file_facts: &'a TsFileFacts,
    pub(super) imports: FxHashMap<&'a str, &'a ImportedBinding>,
    pub(super) calls_by_span: FxHashMap<(u32, u32), &'a ValueCall>,
    pub(super) sites_by_key: FxHashMap<CallKey<'a>, Vec<&'a ResolvedCallSite>>,
    pub(super) builtin_construct_offsets: FxHashSet<u32>,
    pub(super) params: FxHashSet<(usize, &'a str)>,
    pub(super) callback_params: FxHashSet<(CallableId, usize, &'a str)>,
    pub(super) publisher_typed_params: FxHashSet<(usize, &'a str)>,
    pub(super) local_bindings: FxHashMap<(usize, &'a str), &'a ValueExpression>,
    pub(super) iteration_bindings: FxHashMap<(usize, &'a str), &'a ValueExpression>,
    pub(super) local_callables: FxHashMap<(usize, &'a str), CallableId>,
    pub(super) structural_params: FxHashSet<(usize, &'a str)>,
    pub(super) invalidated_locals: FxHashSet<(usize, &'a str)>,
    pub(super) written_members: FxHashSet<(usize, &'a str, Option<&'a str>)>,
}

impl FileEnv<'_> {
    pub(super) fn is_tagged(&self, call: &ValueCall) -> bool {
        self.file_facts
            .value_flow
            .tagged_templates
            .contains_key(&(call.offset, call.end))
    }

    pub(super) fn local_callable(&self, scope: usize, name: &str) -> Option<CallableId> {
        self.local_callables.get(&(scope, name)).copied()
    }

    pub(super) fn has_member_write(&self, scope: usize, name: &str, method: &str) -> bool {
        self.written_members.contains(&(scope, name, Some(method)))
            || self.written_members.contains(&(scope, name, None))
    }
}

pub(super) fn is_callback_param_handle(
    call: Option<&ValueCall>,
    tx_owner: CallableId,
    params: &FxHashSet<(CallableId, usize, &str)>,
) -> bool {
    let Some(call) = call else {
        return false;
    };
    let receiver = match &call.callee {
        ValueExpression::Named { .. } => &call.callee,
        ValueExpression::Member(inner, _) => inner.as_ref(),
        _ => return false,
    };
    matches!(
        receiver,
        ValueExpression::Named { name, scope: Some(scope), .. }
            if params.contains(&(tx_owner, *scope, name.as_str()))
    )
}

pub(super) fn is_param_expr(
    mut receiver: &ValueExpression,
    tx_owner: CallableId,
    params: &FxHashSet<(CallableId, usize, &str)>,
) -> bool {
    while let ValueExpression::Member(inner, _) = receiver {
        receiver = inner.as_ref();
    }
    matches!(
        receiver,
        ValueExpression::Named { name, scope: Some(scope), .. }
            if params.contains(&(tx_owner, *scope, name.as_str()))
    )
}

pub(super) fn classify_builtin_site(
    env: &FileEnv<'_>,
    site: &ResolvedCallSite,
    call: &ValueCall,
    tx_owner: CallableId,
    depth: u8,
) -> Option<BuiltinCall> {
    let ValueExpression::Member(receiver, method) = &call.callee else {
        return None;
    };
    if env.is_tagged(call) {
        return matches!(
            receiver.as_ref(),
            ValueExpression::Named { name, scope: None, .. }
                if name == "String" && method == "raw" && !env.imports.contains_key("String")
        )
        .then(|| builtins::global_static_call("String", "raw"))
        .flatten();
    }
    if let ValueExpression::Named {
        name, scope: None, ..
    } = receiver.as_ref()
    {
        if !env.imports.contains_key(name.as_str()) {
            return builtins::global_static_call(name, method);
        }
    }
    if let ValueExpression::Named { name, scope: Some(scope), .. } = receiver.as_ref() {
        if env.structural_params.contains(&(*scope, name.as_str()))
            || env.has_member_write(*scope, name, method)
        {
            return None;
        }
    }
    let builtin = builtins::proto_method_call(method)?;
    if returned_or_typed_from_publisher(env, site) {
        return None;
    }
    if env.file_facts.builtin_receiver_calls.contains(&call.offset) {
        return Some(builtin);
    }
    is_local_or_builtin_receiver(env, receiver, method, call.offset, tx_owner, depth).then_some(builtin)
}

fn returned_or_typed_from_publisher(env: &FileEnv<'_>, site: &ResolvedCallSite) -> bool {
    let mods = &env.ctx.targets.publisher_modules;
    site.returned_member_from
        .iter()
        .map(|(m, _, _)| m.as_str())
        .chain(site.typed_from.iter().map(|(m, _)| m.as_str()))
        .any(|m| reexports::reaches_module(env.ctx, env.file, m, None, mods))
}

fn is_local_or_builtin_receiver(
    env: &FileEnv<'_>,
    expr: &ValueExpression,
    method: &str,
    call_offset: u32,
    tx_owner: CallableId,
    depth: u8,
) -> bool {
    if depth > 4 {
        return false;
    }
    match expr {
        ValueExpression::Named { name, scope: Some(scope), .. } => {
            local::is_clean_scoped_local(env, *scope, name, method, call_offset, tx_owner, depth)
        }
        ValueExpression::Named { name, scope: None, .. } => {
            !env.imports.contains_key(name.as_str())
                && builtins::GLOBAL_STATIC_OBJECTS.contains(&name.as_str())
        }
        ValueExpression::Member(inner, _) => {
            is_local_or_builtin_receiver(env, inner, method, call_offset, tx_owner, depth)
        }
        ValueExpression::Array(..) => true,
        ValueExpression::Awaited(inner) => {
            is_local_or_builtin_receiver(env, inner, method, call_offset, tx_owner, depth)
        }
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            [left.as_ref(), right.as_ref()].into_iter().all(|arm| {
                matches!(arm, ValueExpression::Null | ValueExpression::Undefined)
                    || is_local_or_builtin_receiver(env, arm, method, call_offset, tx_owner, depth)
            })
        }
        ValueExpression::Derived(parts) => !parts.iter().any(|p| expr_is_import(p, &env.imports)),
        ValueExpression::Call(start, end) => {
            local::inner_call_is_clean(env, *start, *end, tx_owner, depth)
        }
        ValueExpression::Unknown => env.builtin_construct_offsets.contains(&call_offset),
        _ => false,
    }
}

fn expr_is_import(mut expr: &ValueExpression, imports: &FxHashMap<&str, &ImportedBinding>) -> bool {
    while let ValueExpression::Member(inner, _) = expr {
        expr = inner.as_ref();
    }
    matches!(expr, ValueExpression::Named { name, scope: None | Some(0), .. } if imports.contains_key(name.as_str()))
}
