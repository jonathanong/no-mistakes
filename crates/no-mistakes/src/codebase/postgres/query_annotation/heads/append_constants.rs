//! Per-binding first unsafe use; later escapes cannot invalidate an earlier call.
use super::{syntax::Syntax, Span};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
mod uses;

#[derive(Default)]
pub(super) struct Safety {
    limits: FxHashMap<Span, u32>,
    legacy_limits: FxHashMap<Span, u32>,
    first_append: FxHashMap<Span, u32>,
    first_opaque_append: FxHashMap<Span, u32>,
    repeated_calls: FxHashSet<Span>,
    conditional_limits: FxHashMap<Span, u32>,
    pending_by_declaration: FxHashMap<Span, Vec<(u32, Span)>>,
    closed_prefix: FxHashSet<Span>,
}
impl Safety {
    pub fn new(
        syntax: &Syntax,
        module: &TypeScriptModuleFacts,
        executors: &FxHashSet<Span>,
        trusted_roots: &FxHashSet<Span>,
        effects: &super::effects::Prepared,
        closed_prefix: FxHashSet<Span>,
    ) -> Self {
        let allowed = uses::collect(syntax, executors, effects);
        let pending = effects
            .pending
            .iter()
            .map(|call| (call.builder, call.span))
            .collect::<FxHashMap<_, _>>();
        let mut result = Self {
            repeated_calls: syntax.repeated_calls.clone(),
            closed_prefix,
            ..Default::default()
        };
        let roots = module
            .scopes
            .iter()
            .filter(|scope| scope.parent_id.is_none())
            .map(|scope| scope.id)
            .collect::<FxHashSet<_>>();
        let exports = module
            .exports
            .iter()
            .filter(|export| export.specifier.is_none())
            .map(|export| export.local.as_str())
            .collect::<FxHashSet<_>>();
        let finite_roots = syntax
            .finite_roots
            .iter()
            .map(|root| root.declaration)
            .collect::<FxHashSet<_>>();
        for binding in &module.bindings {
            let declaration = (binding.span.start, binding.span.end);
            if !binding.runtime
                || binding.imported
                || !(finite_roots.contains(&declaration)
                    || syntax
                        .constants
                        .get(&declaration)
                        .is_some_and(|head| head.is_constant()))
            {
                continue;
            }
            let context = syntax.contexts.get(&declaration);
            let trusted_root = trusted_roots.contains(&declaration);
            let mut limit = u32::MAX;
            let mut legacy_limit = u32::MAX;
            let mut conditional_limit = u32::MAX;
            if roots.contains(&binding.scope_id) && exports.contains(binding.name.as_str()) {
                limit = 0;
                legacy_limit = 0;
                conditional_limit = 0;
            }
            for usage in binding.references.iter().filter(|usage| usage.runtime) {
                let span = (usage.span.start, usage.span.end);
                let append = syntax.append_receivers.contains(&span);
                if append {
                    result
                        .first_append
                        .entry(declaration)
                        .and_modify(|first| *first = (*first).min(span.0))
                        .or_insert(span.0);
                }
                if trusted_root && syntax.prefix_discarded_appends.contains(&span) {
                    result
                        .first_opaque_append
                        .entry(declaration)
                        .and_modify(|first| *first = (*first).min(span.0))
                        .or_insert(span.0);
                }
                // Captured code may execute before its textual position. Never
                // use textual ordering to excuse a cross-function reference.
                if syntax.contexts.get(&span) != context {
                    limit = 0;
                    legacy_limit = 0;
                    conditional_limit = 0;
                } else if !syntax.discarded_appends.contains(&span)
                    && !(trusted_root && syntax.prefix_discarded_appends.contains(&span))
                    && !allowed.ordinary.contains(&span)
                    && !(trusted_root && allowed.prefix.contains(&span))
                {
                    legacy_limit = legacy_limit.min(span.0);
                    if allowed.low.contains(&span) {
                        continue;
                    }
                    limit = limit.min(span.0);
                    if trusted_root {
                        if let Some(&call) = pending.get(&span) {
                            result
                                .pending_by_declaration
                                .entry(declaration)
                                .or_default()
                                .push((span.0, call));
                            continue;
                        }
                    }
                    conditional_limit = conditional_limit.min(span.0);
                }
            }
            result.limits.insert(declaration, limit);
            result.legacy_limits.insert(declaration, legacy_limit);
            result
                .conditional_limits
                .insert(declaration, conditional_limit);
        }
        result
    }
    pub fn touched(&self, declaration: Span) -> bool {
        self.first_append.contains_key(&declaration)
    }
    pub fn opaque_before(&self, declaration: Span, call: Span) -> bool {
        self.first_opaque_append
            .get(&declaration)
            .is_some_and(|first| *first < call.0)
    }
    pub fn continued(&self, declaration: Span, call: Span) -> bool {
        self.first_append
            .get(&declaration)
            .is_some_and(|first| *first < call.0 || self.repeated_calls.contains(&call))
    }
    pub fn accepts(&self, declaration: Span, call: Span) -> bool {
        if self.closed_prefix.contains(&declaration) {
            return declaration.1 < call.0;
        }
        declaration.1 < call.0
            && self.limits.get(&declaration).is_some_and(|limit| {
                call.1 <= *limit && (*limit == u32::MAX || !self.repeated_calls.contains(&call))
            })
    }
    pub fn closed_prefix(&self, declaration: Span) -> bool {
        self.closed_prefix.contains(&declaration)
    }
    pub fn accepts_legacy(&self, declaration: Span, call: Span) -> bool {
        declaration.1 < call.0
            && self.legacy_limits.get(&declaration).is_some_and(|limit| {
                call.1 <= *limit && (*limit == u32::MAX || !self.repeated_calls.contains(&call))
            })
    }
    pub fn conditional(&self, declaration: Span, call: Span) -> Option<Vec<Span>> {
        let limit = *self.conditional_limits.get(&declaration)?;
        let repeated = self.repeated_calls.contains(&call);
        if declaration.1 >= call.0 || call.1 > limit || (repeated && limit != u32::MAX) {
            return None;
        }
        let mut required = self
            .pending_by_declaration
            .get(&declaration)?
            .iter()
            .filter(|(start, _)| repeated || *start < call.0)
            .map(|(_, span)| *span)
            .collect::<Vec<_>>();
        required.sort_unstable();
        required.dedup();
        (!required.is_empty()).then_some(required)
    }
}
