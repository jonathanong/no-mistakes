use super::ScopeVisitor;
use crate::fx::FxHashSet;

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn mark_lock_helper_definitions(&mut self) {
        if self.lock_helper_applications.is_empty() {
            return;
        }
        let configured: FxHashSet<_> = self
            .lock_helpers
            .values()
            .flat_map(|helper| &helper.calls)
            .filter(|query| query.option_property.is_some())
            .map(|query| query.span)
            .collect();
        let mut applied = FxHashSet::default();
        let mut keep = vec![true; self.calls.len()];
        for (span, indices) in &self.lock_helper_applications {
            if configured.contains(span)
                || indices
                    .iter()
                    .any(|&index| self.calls[index].transaction_id.is_some())
            {
                applied.insert(*span);
            } else {
                for &index in indices {
                    keep[index] = false;
                }
            }
        }
        for (call, span) in self.calls.iter_mut().zip(&self.call_spans) {
            call.helper_definition = applied.contains(span);
        }
        retain(&mut self.calls, &keep);
        retain(&mut self.call_spans, &keep);
        if self.track_order {
            retain(&mut self.confirmed_order, &keep);
        }
    }
}

fn retain<T>(values: &mut Vec<T>, keep: &[bool]) {
    let mut index = 0;
    values.retain(|_| {
        let result = keep[index];
        index += 1;
        result
    });
}
