//! A pending effect still needs the exact stable runtime callee at its use site.
use super::{Bindings, Span, Syntax};
use crate::fx::{FxHashMap, FxHashSet};

pub(super) struct PendingUses {
    pub references: FxHashMap<Span, Span>,
    calls: Option<FxHashMap<Span, (Span, Span)>>,
    stable: FxHashMap<usize, bool>,
    visits: u64,
}
impl PendingUses {
    pub fn new(bindings: &Bindings<'_>, executors: &FxHashSet<Span>) -> Self {
        let mut result = Self {
            references: FxHashMap::default(),
            calls: None,
            stable: FxHashMap::default(),
            visits: 0,
        };
        // Confirmed sinks already have a proof; do not index them as effects.
        result.references = bindings
            .effects()
            .pending
            .iter()
            .filter(|pending| !executors.contains(&pending.span))
            .map(|pending| (pending.builder, pending.span))
            .collect();
        result
    }
    pub fn accepts(
        &mut self,
        span: Span,
        reference: Span,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
    ) -> bool {
        if !bindings.declaration_safe(span) {
            return false;
        }
        // Build once only after a captured owner's complete census succeeds.
        let calls = self.calls.get_or_insert_with(|| {
            syntax
                .effect_calls
                .iter()
                .filter(|call| call.position == 0)
                .map(|call| (call.span, (call.callee, call.builder)))
                .collect()
        });
        let Some((callee, _)) = calls
            .get(&span)
            .filter(|(_, builder)| *builder == reference)
        else {
            return false;
        };
        let Some(owner) = bindings.owner(*callee) else {
            return false;
        };
        let visits = &mut self.visits;
        *self.stable.entry(owner.id).or_insert_with(|| {
            owner.references.iter().all(|usage| {
                *visits += 1;
                let span = (usage.span.start, usage.span.end);
                bindings.declaration_safe(span)
                    && !syntax.assignments.writes.contains_key(&span)
                    && !syntax.assignments.unsupported_writes.contains(&span)
            })
        })
    }
    pub fn report(&self, observer: &crate::diagnostics::InvocationObserver) {
        if self.calls.is_some() {
            observer.increment("postgres.annotation.captured_pending_call_indexes", 1);
        }
        if self.stable.is_empty() {
            return;
        }
        observer.increment(
            "postgres.annotation.captured_pending_callee_censuses",
            self.stable.len() as u64,
        );
        observer.increment(
            "postgres.annotation.captured_pending_callee_reference_visits",
            self.visits,
        );
    }
}
