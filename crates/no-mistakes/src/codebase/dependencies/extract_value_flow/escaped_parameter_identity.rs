use super::*;

#[derive(Default)]
pub(super) struct Epochs {
    writes: std::collections::BTreeMap<u32, Option<CallableId>>,
    foreign: bool,
}
impl ImportCollector {
    pub(in super::super::super) fn note_guarded_parameter_write(
        &mut self,
        scope: usize,
        name: &str,
        offset: u32,
        completed_at: u32,
    ) {
        if !self.collect_value_flow {
            return;
        }
        let Some(owner) = self
            .value_parameter_indexes
            .get(&scope)
            .and_then(|names| names.get(name))
            .and_then(|indexes| indexes.first())
            .and_then(|index| self.value_flow.parameters.get(*index))
            .map(|parameter| parameter.owner)
        else {
            return;
        };
        let context = self.current_function_id();
        let completion = self
            .receiver_escapes
            .parameter_completion
            .filter(|(start, _)| *start == offset)
            .map(|(_, end)| end)
            .or_else(|| (completed_at > offset).then_some(completed_at));
        let certain = !self.conditional_execution
            && !self.receiver_write_is_conditional
            && self.repeat_depth <= self.call_repeat_base;
        let epochs = self
            .receiver_escapes
            .parameter_epochs
            .entry(scope)
            .or_default()
            .entry(name.to_owned())
            .or_default();
        epochs.foreign |= context != Some(owner);
        // Unmatched or conditional writes never establish a known parameter epoch.
        epochs.writes.insert(
            completion.unwrap_or(offset),
            completion.filter(|_| certain).and(context),
        );
    }
    pub(in super::super) fn parameter_capture_writes_covered(
        &self,
        key: &(usize, String),
        completions: &FxHashSet<u32>,
    ) -> bool {
        self.receiver_escapes
            .parameter_epochs
            .get(&key.0)
            .and_then(|names| names.get(&key.1))
            .is_none_or(|epochs| {
                !epochs.foreign && epochs.writes.keys().all(|at| completions.contains(at))
            })
    }
    pub(in super::super) fn guarded_parameter_epoch(
        &self,
        binding: &(usize, String),
        owner: CallableId,
        offset: u32,
        context: Option<CallableId>,
    ) -> Option<u32> {
        if context != Some(owner) {
            return None;
        }
        let Some(epochs) = self
            .receiver_escapes
            .parameter_epochs
            .get(&binding.0)
            .and_then(|names| names.get(&binding.1))
        else {
            return Some(0);
        };
        if epochs.foreign {
            return None;
        }
        match epochs.writes.range(..=offset).next_back() {
            None => Some(0),
            Some((at, source)) => (*source == context).then_some(*at),
        }
    }
}

#[cfg(test)]
#[path = "parameter_capture_epoch_tests.rs"]
mod capture_tests;
