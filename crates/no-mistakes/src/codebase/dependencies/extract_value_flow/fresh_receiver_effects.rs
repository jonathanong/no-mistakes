use super::*;
use std::collections::BTreeSet;

#[derive(Default)]
pub(in super::super) struct FreshReceiverEffects {
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) rhs_probe:
        Option<std::rc::Rc<std::cell::RefCell<inert_receiver_rhs::tests_support::Probe>>>,
    pub active: bool,
    pub epoch: u64,
    pub descriptor_call: Option<u32>,
    pub(super) phases:
        Option<std::rc::Rc<std::cell::RefCell<active_reference_phases::ActiveReferencePhases>>>,
    #[cfg(feature = "test-instrumentation")]
    identity_visits: std::cell::Cell<usize>,
    #[cfg(feature = "test-instrumentation")]
    rebind_calls: usize,
    #[cfg(feature = "test-instrumentation")]
    rebind_name_allocations: usize,
    #[cfg(feature = "test-instrumentation")]
    pub(super) rhs_visits: usize,
    captures: Option<Box<capture_reads::CaptureInterner>>,
    calls: Option<Box<FreshCalls>>,
    rebound: Option<Box<FxHashMap<usize, FxHashSet<String>>>>,
}
#[derive(Default)]
struct FreshCalls(FxHashMap<(Option<CallableId>, u64), BTreeSet<u32>>);
impl FreshReceiverEffects {
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn record_counts(&self, facts: &mut ValueFlowFacts) {
        facts.reference_rhs_visits = self.rhs_visits;
        facts.reference_rebind_calls = self.rebind_calls;
        facts.reference_rebind_name_allocations = self.rebind_name_allocations;
        facts.reference_identity_visits = self.identity_visits();
        (
            facts.reference_capture_nodes,
            facts.reference_capture_logical_tokens,
            facts.reference_capture_indexes,
        ) = self.capture_counts();
        facts.reference_call_indexes = usize::from(self.calls.is_some());
        facts.reference_call_entries = self
            .calls
            .as_ref()
            .map_or(0, |calls| calls.0.values().map(BTreeSet::len).sum());
    }
    pub(super) fn note_rhs_visit(&mut self) {
        #[cfg(feature = "test-instrumentation")]
        {
            self.rhs_visits += 1;
        }
    }
    pub(super) fn capture(
        &mut self,
        reads: &[nested::ReceiverRead],
    ) -> capture_reads::CaptureReads {
        self.captures
            .get_or_insert_with(Default::default)
            .capture(reads)
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn capture_counts(&self) -> (usize, usize, usize) {
        self.captures.as_ref().map_or((0, 0, 0), |captures| {
            (captures.nodes(), captures.logical_tokens, 1)
        })
    }
    pub(super) fn note_identity_visit(&self) {
        #[cfg(feature = "test-instrumentation")]
        self.identity_visits.set(self.identity_visits.get() + 1);
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn identity_visits(&self) -> usize {
        self.identity_visits.get()
    }
    pub(super) fn call(&mut self, offset: u32, context: Option<CallableId>) {
        if self.active || self.captures.is_some() {
            self.calls
                .get_or_insert_with(Default::default)
                .0
                .entry((context, self.epoch))
                .or_default()
                .insert(offset);
        }
    }
    pub(super) fn called_between(
        &self,
        left: nested::ReceiverRead,
        right: nested::ReceiverRead,
    ) -> bool {
        let (Some((left, context)), Some((right, other))) = (left, right) else {
            return true;
        };
        let Some(calls) = &self.calls else {
            return false;
        };
        if context != other {
            return calls.0.contains_key(&(context, self.epoch))
                || calls.0.contains_key(&(other, self.epoch));
        }
        calls
            .0
            .get(&(context, self.epoch))
            .is_some_and(|events| identity_metadata::between(events, left, right))
    }
    pub(super) fn rebound(&self, scope: usize, name: &str) -> bool {
        self.rebound
            .as_ref()
            .and_then(|scopes| scopes.get(&scope))
            .is_some_and(|names| names.contains(name))
    }
    pub(super) fn rebind(&mut self, scope: usize, name: &str) {
        #[cfg(feature = "test-instrumentation")]
        {
            self.rebind_calls += 1;
        }
        let names = self
            .rebound
            .get_or_insert_with(Default::default)
            .entry(scope)
            .or_default();
        if !names.contains(name) {
            #[cfg(feature = "test-instrumentation")]
            {
                self.rebind_name_allocations += 1;
            }
            names.insert(name.to_string());
        }
    }
}
impl ImportCollector {
    pub(in super::super) fn visit_receiver_call(&mut self, call: &CallExpression<'_>) {
        self.record_unknown_fresh_call(call.span.end);
        visit_call_expression_per_item(self, call);
        self.record_descriptor_call(call.span.end);
    }
    pub(in super::super) fn visit_receiver_tagged(
        &mut self,
        tagged: &TaggedTemplateExpression<'_>,
    ) {
        self.record_unknown_fresh_call(tagged.span.end);
        self.visit_value_tagged(tagged);
        self.record_descriptor_call(tagged.span.end);
    }
    pub(super) fn receiver_acquisition_offset(&self, offset: u32) -> u32 {
        self.assignment_target_start
            .filter(|_| self.destructuring_phase_owner == Some(self.current_function_id()))
            .filter(|target| {
                self.fresh_receiver_effects
                    .phases
                    .as_ref()
                    .and_then(|phases| phases.borrow().capture_phase(Some(*target)))
                    .is_some()
            })
            .unwrap_or(offset)
    }
    pub(in super::super) fn record_descriptor_call(&mut self, offset: u32) {
        if self.collect_value_flow {
            self.fresh_receiver_effects
                .call(offset, self.current_function_id());
            self.fresh_receiver_effects.descriptor_call = Some(
                self.fresh_receiver_effects
                    .descriptor_call
                    .map_or(offset, |old| old.max(offset)),
            );
        }
    }
    pub(in super::super) fn record_unknown_fresh_call(&mut self, offset: u32) {
        self.fresh_receiver_effects
            .call(offset, self.current_function_id());
    }
}

#[cfg(test)]
#[path = "fresh_receiver_effects_tests.rs"]
mod tests;
