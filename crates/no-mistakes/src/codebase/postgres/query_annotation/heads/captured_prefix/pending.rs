//! A pending effect still needs the exact stable runtime callee at its use site.
use super::{Bindings, Span, Syntax};
use crate::fx::FxHashMap;

pub(super) struct PendingUses {
    pub references: FxHashMap<Span, Span>,
    callees: FxHashMap<Span, Span>,
    stable: FxHashMap<usize, bool>,
    visits: u64,
}
impl PendingUses {
    pub fn new(syntax: &Syntax, bindings: &Bindings<'_>) -> Self {
        let mut result = Self {
            references: FxHashMap::default(),
            callees: FxHashMap::default(),
            stable: FxHashMap::default(),
            visits: 0,
        };
        if bindings.effects().pending.is_empty() {
            return result;
        }
        let calls = syntax
            .effect_calls
            .iter()
            .filter(|call| call.position == 0)
            .map(|call| (call.span, call))
            .collect::<FxHashMap<_, _>>();
        for pending in &bindings.effects().pending {
            let Some(call) = calls
                .get(&pending.span)
                .filter(|call| call.builder == pending.builder)
            else {
                continue;
            };
            result.references.insert(pending.builder, pending.span);
            result.callees.insert(pending.span, call.callee);
        }
        result
    }
    pub fn accepts(&mut self, span: Span, syntax: &Syntax, bindings: &Bindings<'_>) -> bool {
        if !bindings.declaration_safe(span) {
            return false;
        }
        let Some(owner) = self
            .callees
            .get(&span)
            .and_then(|callee| bindings.owner(*callee))
        else {
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
