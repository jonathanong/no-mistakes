use super::builtins::{self, BuiltinCall};
use super::options::Api;
use super::receivers::{is_param_expr, FileEnv};
use super::reexports::{self, ResolvedApi};
use super::{finding, unanalyzable};
use crate::codebase::dependencies::extract::{CallableId, ValueCall, ValueExpression};
use crate::codebase::dependencies::graph::ResolvedCallSite;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::RuleFinding;
use crate::fx::FxHashSet;

pub(super) struct CallbackWalk<'a> {
    pub(super) worklist: Vec<(CallableId, CallableId)>,
    pub(super) visited: FxHashSet<(CallableId, CallableId)>,
    pub(super) reported: FxHashSet<(u32, String)>,
    pub(super) diagnostics: &'a mut AnalysisDiagnostics,
}

impl CallbackWalk<'_> {
    pub(super) fn report_publish(&mut self, env: &FileEnv<'_>, site: &ResolvedCallSite, api: Api<'_>) {
        self.push_unique(site, finding(env.ctx, site, api));
    }

    pub(super) fn report_unanalyzable(
        &mut self,
        env: &FileEnv<'_>,
        site: &ResolvedCallSite,
        reason: &str,
    ) {
        let item = unanalyzable(env.ctx, site, reason);
        if self.mark(site, &item) {
            self.diagnostics.unanalyzable(env.ctx.policy, item);
        }
    }

    fn push_unique(&mut self, site: &ResolvedCallSite, item: RuleFinding) {
        if self.mark(site, &item) {
            self.diagnostics.findings.push(item);
        }
    }

    fn mark(&mut self, site: &ResolvedCallSite, item: &RuleFinding) -> bool {
        self.reported.insert((site.offset, item.message.clone()))
    }

    pub(super) fn inspect_builtin(
        &mut self,
        env: &FileEnv<'_>,
        site: &ResolvedCallSite,
        call: &ValueCall,
        builtin: BuiltinCall,
        tx_owner: CallableId,
    ) {
        for &index in builtin.callbacks {
            if call.spread_at.is_some_and(|spread| spread <= index) {
                self.report_unanalyzable(env, site,
                    "built-in call inside transaction callback has a spread argument at or before its callback slot; pass arguments positionally so callbacks can be checked");
                return;
            }
            if let Some(arg) = call.arguments.get(index) {
                self.inspect_arg(env, site, arg, builtin.optional_non_callable, tx_owner, 0);
            }
        }
    }

    fn inspect_arg(
        &mut self,
        env: &FileEnv<'_>,
        site: &ResolvedCallSite,
        arg: &ValueExpression,
        optional_non_callable: bool,
        tx_owner: CallableId,
        depth: u8,
    ) {
        if depth > 4 {
            self.emit_unprovable(env, site);
            return;
        }
        match arg {
            ValueExpression::Callable(cb) => {
                if env.file_facts.value_flow.generator_callables.contains(cb) {
                    self.report_unanalyzable(env, site,
                        "built-in callback inside transaction is a generator function, whose execution timing cannot be proved; use a regular or async inline callback");
                } else if self.visited.insert((*cb, tx_owner)) {
                    self.worklist.push((*cb, tx_owner));
                }
            }
            ValueExpression::Undefined
            | ValueExpression::Null
            | ValueExpression::Array(..)
            | ValueExpression::Object(..) => {}
            ValueExpression::Unknown if optional_non_callable => {}
            ValueExpression::Derived(parts) if optional_non_callable => {
                for part in parts.iter() {
                    self.inspect_arg(env, site, part, true, tx_owner, depth + 1);
                }
            }
            ValueExpression::Named { .. } | ValueExpression::Member(..) => {
                self.inspect_named_or_member(env, site, arg, optional_non_callable, tx_owner, depth);
            }
            _ => self.emit_unprovable(env, site),
        }
    }

    fn inspect_named_or_member(
        &mut self,
        env: &FileEnv<'_>,
        site: &ResolvedCallSite,
        arg: &ValueExpression,
        optional_non_callable: bool,
        tx_owner: CallableId,
        depth: u8,
    ) {
        if is_param_expr(arg, tx_owner, &env.callback_params)
            || builtins::is_builtin_global_fn(arg, &env.imports)
        {
            return;
        }
        if let Some(imported) = resolve_imported_expr(env, arg) {
            match imported {
                ResolvedApi::Target(spec, export) => {
                    self.report_publish(env, site, (spec.as_str(), export.as_str()));
                }
                ResolvedApi::Unanalyzable => self.emit_unprovable(env, site),
                ResolvedApi::Other => {}
            }
            return;
        }
        if let ValueExpression::Named { name, scope: Some(scope), .. } = arg {
            if env.invalidated_locals.contains(&(*scope, name.as_str())) {
                if !optional_non_callable {
                    self.emit_unprovable(env, site);
                }
                return;
            }
            if let Some(cb) = env.local_callable(*scope, name) {
                self.inspect_arg(env, site, &ValueExpression::Callable(cb), optional_non_callable, tx_owner, depth + 1);
                return;
            }
            if let Some(bound) = env.local_bindings.get(&(*scope, name.as_str())) {
                if !matches!(bound, ValueExpression::Unknown) {
                    self.inspect_arg(env, site, bound, optional_non_callable, tx_owner, depth + 1);
                    return;
                }
            }
        }
        if !optional_non_callable {
            self.emit_unprovable(env, site);
        }
    }

    fn emit_unprovable(&mut self, env: &FileEnv<'_>, site: &ResolvedCallSite) {
        self.report_unanalyzable(env, site,
            "built-in callback inside transaction cannot be proved not to publish; pass an inline callback or move publishes after commit");
    }
}

fn resolve_imported_expr(env: &FileEnv<'_>, expr: &ValueExpression) -> Option<ResolvedApi> {
    let mut parts = Vec::new();
    let mut cur = expr;
    while let ValueExpression::Member(inner, prop) = cur {
        parts.push(prop.as_str());
        cur = inner.as_ref();
    }
    let ValueExpression::Named { name, scope: None | Some(0), .. } = cur else {
        return None;
    };
    let binding = env.imports.get(name.as_str())?;
    parts.reverse();
    let suffix = (!parts.is_empty()).then(|| parts.join("."));
    reexports::resolve_binding_api(env.ctx, env.file, binding, suffix.as_deref(), |m, e| {
        env.ctx.targets.publishers.contains(&(m, e))
    })
}
