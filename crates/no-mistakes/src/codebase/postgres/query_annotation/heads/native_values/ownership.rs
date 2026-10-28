//! Closed native constructor identity is separate from observation output values.
use super::Span;
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::FxHashSet;

fn native_ids(module: &TypeScriptModuleFacts) -> FxHashSet<usize> {
    module
        .imports
        .iter()
        .filter(|import| !import.type_only && import.specifier == "sql-template-strings")
        .flat_map(|import| &import.bindings)
        .filter(|binding| !binding.type_only)
        .map(|binding| binding.binding_id)
        .collect()
}
pub(in super::super) fn extend_wanted(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    wanted: &mut FxHashSet<Span>,
) {
    if !syntax.native_values.demanded() {
        return;
    }
    let native = native_ids(module);
    for binding in &module.bindings {
        if native.contains(&binding.id) {
            wanted.extend(
                binding
                    .references
                    .iter()
                    .filter(|usage| usage.runtime)
                    .map(|usage| (usage.span.start, usage.span.end)),
            );
        }
    }
}
pub(super) struct Ownership<'a> {
    roots: FxHashSet<usize>,
    exports: FxHashSet<&'a str>,
    native: Vec<&'a ModuleBinding>,
    complete: bool,
    safe: Option<bool>,
}
impl<'a> Ownership<'a> {
    pub fn new(module: &'a TypeScriptModuleFacts) -> Self {
        let ids = native_ids(module);
        let native = module
            .bindings
            .iter()
            .filter(|binding| binding.runtime && ids.contains(&binding.id))
            .collect::<Vec<_>>();
        let complete = native.len() == ids.len()
            && !module.exports.iter().any(|export| {
                !export.type_only && export.specifier.as_deref() == Some("sql-template-strings")
            })
            && !module
                .loads
                .iter()
                .any(|load| load.specifier == "sql-template-strings");
        Self {
            roots: module
                .scopes
                .iter()
                .filter(|scope| scope.parent_id.is_none())
                .map(|scope| scope.id)
                .collect(),
            exports: module
                .exports
                .iter()
                .filter(|export| !export.type_only && export.specifier.is_none())
                .map(|export| export.local.as_str())
                .collect(),
            native,
            complete,
            safe: None,
        }
    }
    pub fn module_heap(&self, owner: &ModuleBinding) -> bool {
        self.roots.contains(&owner.scope_id)
    }
    pub fn constructor(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        reference: Span,
    ) -> bool {
        let Some(root) = bindings.owner(reference) else {
            return false;
        };
        if !bindings.trusted_import(root.id) {
            return false;
        }
        if self.safe.is_none() {
            let mut visits = 0;
            let safe = self.complete
                && !self.native.is_empty()
                && self.native.iter().all(|owner| {
                    visits += 1;
                    owner.imported
                        && bindings.trusted_import(owner.id)
                        && !(self.module_heap(owner) && self.exports.contains(owner.name.as_str()))
                        && bindings.declaration_safe((owner.span.start, owner.span.end))
                        && owner
                            .references
                            .iter()
                            .filter(|usage| usage.runtime)
                            .all(|usage| {
                                let span = (usage.span.start, usage.span.end);
                                syntax.native_values.tags.contains(&span)
                                    && bindings
                                        .owner(span)
                                        .is_some_and(|actual| actual.id == owner.id)
                                    && !syntax.assignments.writes.contains_key(&span)
                                    && !syntax.assignments.unsupported_writes.contains(&span)
                            })
                });
            self.safe = Some(safe);
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment(
                    "postgres.annotation.native_value_constructor_censuses",
                    visits,
                );
            }
        }
        self.safe == Some(true)
    }
}
