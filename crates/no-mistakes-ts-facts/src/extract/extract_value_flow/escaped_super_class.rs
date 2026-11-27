use super::*;

pub(super) fn super_class(collector: &ImportCollector) -> Option<(CallableId, usize)> {
    let class_id = this_class(collector)?;
    // Captured at the extends clause; method-local bindings cannot shadow it.
    let scope = *collector.receiver_escapes.base_scopes.get(&class_id)?;
    let base = collector.class_local_bases.get(&class_id)?;
    Some((collector.class_id_for_binding(scope, base)?, scope))
}

impl ImportCollector {
    pub(in super::super::super::super) fn record_class_local_base(
        &mut self,
        class_id: CallableId,
        base: String,
    ) {
        if self.collect_value_flow {
            self.receiver_escapes
                .base_scopes
                .insert(class_id, self.current_lexical_scope_id());
        }
        self.class_local_bases.insert(class_id, base);
    }
}
