#[macro_use]
mod scope_visits;
mod callback_transactions;
mod class_receivers;
mod delayed;
mod object_params;
mod receiver_types;
mod type_shadows;
mod undeclared;
mod visitor;
use super::super::relative::PendingRelativeSpan;
use super::candidates::ImportClassification;
use super::owners::{self, NameHit};
use super::ScopedExecutors;
use oxc_span::Span;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct ScopeCollector {
    delayed: delayed::Bindings,
    pub(super) track_resource_transactions: bool,
    factories: HashSet<String>,
    types: HashSet<String>,
    factory_shadows: HashMap<String, Vec<Span>>,
    undeclared: HashMap<String, super::super::UndeclaredExecutorType>,
    provisional_factories: HashMap<String, Vec<u32>>,
    provisional_types: HashMap<String, Vec<u32>>,
    scopes: Vec<Span>,
    type_shadows: Vec<(String, Span)>,
    classes: Vec<Span>,
    class_methods: HashSet<u32>,
    pending_annotations: Vec<type_shadows::PendingAnnotation>,
    pub(super) found: ScopedExecutors,
    pub(super) spans: Vec<PendingRelativeSpan>,
}
impl ScopeCollector {
    pub(super) fn needs_walk(&self) -> bool {
        !self.undeclared.is_empty()
            || !self.factories.is_empty()
            || !self.types.is_empty()
            || !self.provisional_factories.is_empty()
            || !self.provisional_types.is_empty()
    }

    fn with_scope(&mut self, span: Span, walk: impl FnOnce(&mut Self)) {
        self.scopes.push(span);
        walk(self);
        self.scopes.pop();
    }
}

#[cfg(test)]
mod delayed_tests;
