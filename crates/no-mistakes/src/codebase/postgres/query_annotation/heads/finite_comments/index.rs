//! Request-local finite-comment joins, built once from canonical prepared facts.
use super::{CallSite, Function, Select};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, Span,
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
mod roots;
#[derive(Default)]
pub(super) struct Work {
    pub domains: u64,
    pub stable: u64,
    pub references: u64,
    pub endpoints: u64,
}
pub(super) struct Index<'a> {
    pub functions: FxHashMap<Span, &'a Function>,
    pub selects: FxHashMap<Span, &'a Select>,
    pub declarations: FxHashMap<Span, Option<&'a ModuleBinding>>,
    pub calls: FxHashMap<usize, Vec<&'a CallSite>>,
    pub endpoints: FxHashMap<Span, Vec<(Span, Span)>>,
    pub domains: FxHashMap<usize, Option<Vec<String>>>,
    exports: FxHashSet<&'a str>,
    stable: FxHashMap<usize, bool>,
    pub work: Work,
}
impl<'a> Index<'a> {
    pub fn new(
        syntax: &'a Syntax,
        bindings: &Bindings<'_>,
        module: &'a TypeScriptModuleFacts,
        executors: &FxHashSet<Span>,
    ) -> Self {
        let mut result = Self {
            functions: FxHashMap::default(),
            selects: FxHashMap::default(),
            declarations: FxHashMap::default(),
            calls: FxHashMap::default(),
            endpoints: FxHashMap::default(),
            domains: FxHashMap::default(),
            exports: FxHashSet::default(),
            stable: FxHashMap::default(),
            work: Work::default(),
        };
        result.exports.extend(
            module
                .exports
                .iter()
                .filter(|export| export.specifier.is_none() && !export.type_only)
                .map(|export| export.local.as_str()),
        );
        let [functions, selects, declarations, helpers, calls] =
            result.build_roots(syntax, bindings, module, executors);
        if let Some(observer) = crate::diagnostics::current() {
            for (metric, count) in [
                ("postgres.annotation.finite_index_builds", 1),
                (
                    "postgres.annotation.finite_index_function_visits",
                    functions,
                ),
                ("postgres.annotation.finite_index_select_visits", selects),
                (
                    "postgres.annotation.finite_index_binding_visits",
                    declarations,
                ),
                (
                    "postgres.annotation.finite_index_export_visits",
                    module.exports.len(),
                ),
                (
                    "postgres.annotation.finite_index_helper_call_visits",
                    helpers,
                ),
                ("postgres.annotation.finite_index_call_visits", calls),
            ] {
                observer.increment(metric, count as u64);
            }
        }
        result
    }
    pub fn declared(&self, span: Span) -> Option<&'a ModuleBinding> {
        self.declarations.get(&span).copied().flatten()
    }
    pub fn stable(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        binding: &ModuleBinding,
    ) -> bool {
        if let Some(valid) = self.stable.get(&binding.id) {
            return *valid;
        }
        self.work.stable += 1;
        let declaration = (binding.span.start, binding.span.end);
        let valid = bindings.declaration_safe(declaration)
            && !self.exports.contains(binding.name.as_str())
            && binding
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .all(|usage| {
                    self.work.references += 1;
                    let span = (usage.span.start, usage.span.end);
                    bindings
                        .owner(span)
                        .is_some_and(|owner| owner.id == binding.id)
                        && !syntax.assignments.unsupported_writes.contains(&span)
                        && !syntax.assignments.writes.contains_key(&span)
                });
        self.stable.insert(binding.id, valid);
        valid
    }
    pub fn only_references(&mut self, binding: &ModuleBinding, expected: &[Span]) -> bool {
        let mut count = 0;
        for usage in binding.references.iter().filter(|usage| usage.runtime) {
            self.work.references += 1;
            count += 1;
            if count > expected.len() || !expected.contains(&(usage.span.start, usage.span.end)) {
                return false;
            }
        }
        count == expected.len()
    }
    pub fn report(&self) {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(
                "postgres.annotation.finite_helper_domain_censuses",
                self.work.domains,
            );
            observer.increment(
                "postgres.annotation.finite_stability_censuses",
                self.work.stable,
            );
            observer.increment(
                "postgres.annotation.finite_reference_visits",
                self.work.references,
            );
            observer.increment(
                "postgres.annotation.finite_root_endpoint_visits",
                self.work.endpoints,
            );
        }
    }
}
