use super::*;

impl ImportCollector {
    pub(super) fn record_registry_mutation(&mut self, name: &str) {
        self.record_registry_effect(name, false);
    }

    pub(super) fn record_registry_export(&mut self, name: &str) {
        self.record_registry_effect(name, true);
    }

    fn record_registry_effect(&mut self, name: &str, export_only: bool) {
        let Some(scope) = self.callee_binding_scope(name) else {
            return;
        };
        if let Some(existing) = self
            .value_flow
            .registry_mutations
            .iter_mut()
            .find(|mutation| mutation.name == name && mutation.scope == scope)
        {
            existing.export_only &= export_only;
            return;
        }
        self.value_flow.registry_mutations.push(RegistryMutation {
            name: name.to_string(),
            scope,
            export_only,
        });
    }
}
