//! Direct loop templates never demand root/helper join indexes.
use super::Index;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
    Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::FxHashSet;
impl<'a> Index<'a> {
    pub(super) fn build_roots(
        &mut self,
        syntax: &'a Syntax,
        bindings: &Bindings<'_>,
        module: &'a TypeScriptModuleFacts,
        executors: &FxHashSet<Span>,
    ) -> [usize; 5] {
        if syntax.finite_roots.is_empty() {
            return [0; 5];
        }
        let roots = syntax
            .finite_roots
            .iter()
            .map(|root| root.declaration)
            .collect::<FxHashSet<_>>();
        for call in &syntax.calls {
            if !executors.contains(&call.span) {
                continue;
            }
            let Head::Reference(reference) = call.head else {
                continue;
            };
            let Some(owner) = bindings.owner(reference) else {
                continue;
            };
            let declaration = (owner.span.start, owner.span.end);
            if roots.contains(&declaration) {
                self.endpoints
                    .entry(declaration)
                    .or_default()
                    .push((call.span, reference));
            }
        }
        if self.endpoints.is_empty() {
            return [0, 0, 0, 0, syntax.calls.len()];
        }
        for function in &syntax.finite_functions {
            self.functions.entry(function.body).or_insert(function);
        }
        for select in &syntax.finite_selects {
            self.selects.entry(select.body).or_insert(select);
        }
        for binding in module
            .bindings
            .iter()
            .filter(|binding| binding.runtime && !binding.imported)
        {
            self.declarations
                .entry((binding.span.start, binding.span.end))
                .and_modify(|value| *value = None)
                .or_insert(Some(binding));
        }
        for call in &syntax.finite_calls {
            if let Some(owner) = bindings.owner(call.callee) {
                self.calls.entry(owner.id).or_default().push(call);
            }
        }
        [
            syntax.finite_functions.len(),
            syntax.finite_selects.len(),
            module.bindings.len(),
            syntax.finite_calls.len(),
            syntax.calls.len(),
        ]
    }
}
