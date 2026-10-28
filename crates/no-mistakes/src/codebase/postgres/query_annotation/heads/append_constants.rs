//! Per-binding first unsafe use; later escapes cannot invalidate an earlier call.
use super::{
    syntax::{Head, Syntax},
    Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct Safety {
    limits: FxHashMap<Span, u32>,
    first_append: FxHashMap<Span, u32>,
    first_opaque_append: FxHashMap<Span, u32>,
    repeated_calls: FxHashSet<Span>,
}
impl Safety {
    pub fn new(
        syntax: &Syntax,
        module: &TypeScriptModuleFacts,
        executors: &FxHashSet<Span>,
        trusted_roots: &FxHashSet<Span>,
    ) -> Self {
        let allowed = syntax
            .calls
            .iter()
            .filter(|call| executors.contains(&call.span))
            .filter(|call| !matches!(call.head, Head::AppendPrefixReference(_)))
            .filter_map(|call| call.head.reference())
            .collect::<FxHashSet<_>>();
        let prefix_allowed = syntax
            .calls
            .iter()
            .filter(|call| executors.contains(&call.span))
            .filter_map(|call| match call.head {
                Head::AppendPrefixReference(reference) => Some(reference),
                _ => None,
            })
            .collect::<FxHashSet<_>>();
        let mut result = Self {
            repeated_calls: syntax.repeated_calls.clone(),
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
        for binding in &module.bindings {
            let declaration = (binding.span.start, binding.span.end);
            if !binding.runtime
                || binding.imported
                || !syntax
                    .constants
                    .get(&declaration)
                    .is_some_and(|head| head.is_constant())
            {
                continue;
            }
            let context = syntax.contexts.get(&declaration);
            let trusted_root = trusted_roots.contains(&declaration);
            let mut limit = u32::MAX;
            if roots.contains(&binding.scope_id) && exports.contains(binding.name.as_str()) {
                limit = 0;
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
                } else if !syntax.discarded_appends.contains(&span)
                    && !(trusted_root && syntax.prefix_discarded_appends.contains(&span))
                    && !allowed.contains(&span)
                    // Only a trusted append method preserves the receiver prefix.
                    && !(trusted_root && prefix_allowed.contains(&span))
                {
                    limit = limit.min(span.0);
                }
            }
            result.limits.insert(declaration, limit);
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
        declaration.1 < call.0
            && self.limits.get(&declaration).is_some_and(|limit| {
                call.1 <= *limit && (*limit == u32::MAX || !self.repeated_calls.contains(&call))
            })
    }
}
