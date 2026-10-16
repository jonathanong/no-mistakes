use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn parameter(
        &mut self,
        path: &std::path::Path,
        parameter: &ValueParameter,
        index: &CallableFileIndex,
    ) -> FlowValue {
        self.used_parameters
            .insert((path.to_path_buf(), parameter.owner));
        if !self.parameters_enabled {
            return FlowValue::Unknown;
        }
        let key = ParameterKey {
            file: path.to_path_buf(),
            scope: parameter.scope,
            name: parameter.name.clone(),
            invocation_read: self.invocation_read.clone(),
            initializing: self.initializing.clone(),
        };
        let cached = match self.cache.acquire(key) {
            CacheAccess::Ready(value) => value,
            CacheAccess::Cycle => return FlowValue::Unknown,
            CacheAccess::Compute(lease) => {
                let (value, origins, used_parameters) = {
                    #[cfg(all(test, feature = "test-instrumentation"))]
                    let _computation_scope = Self::test_computation_scope();
                    let mut flow = ValueFlowResolution {
                        inputs: self.inputs,
                        facts: self.facts,
                        resolver: self.resolver,
                        indexes: self.indexes,
                        incoming: self.incoming,
                        supers: self.supers,
                        cache: self.cache,
                        parameters_enabled: true,
                        initializing: self.initializing.clone(),
                        used_parameters: fx_set(),
                        visiting: fx_set(),
                        calls_visiting: fx_set(),
                        default_origins: None,
                        acquisition_context: None,
                        invocation_read: self.invocation_read.clone(),
                        invocation_start: self.invocation_start,
                        executing_read: None,
                        module_writer_phases: self.module_writer_phases,
                        writer_dependencies: Default::default(),
                        module_snapshot: None,
                        repeated_initializer: None,
                        member_read: None,
                    };
                    flow.visiting.insert((
                        path.to_path_buf(),
                        parameter.scope,
                        parameter.name.clone(),
                    ));
                    flow.used_parameters
                        .insert((path.to_path_buf(), parameter.owner));
                    let value = flow.compute_parameter(path, parameter, false);
                    // Async callbacks may still carry exact executor metadata even
                    // when ordinary temporal value agreement is unavailable.
                    let contextual = flow
                        .incoming
                        .get(&(path.to_path_buf(), parameter.owner))
                        .is_some_and(|inputs| {
                            inputs.iter().any(|input| input.synthetic_callback)
                                && inputs.iter().any(|input| !input.callback)
                        });
                    let value = if contextual && matches!(value, FlowValue::Unknown) {
                        flow.compute_parameter(path, parameter, true)
                    } else {
                        value
                    };
                    #[cfg(all(test, feature = "test-instrumentation"))]
                    let value = if Self::test_old_annotation_lifetime() {
                        flow.typed_parameter(path, parameter, index, value)
                    } else {
                        value
                    };
                    flow.merge_writer_dependencies();
                    #[cfg(all(test, feature = "test-instrumentation"))]
                    flow.test_retiring_computation_buffers();
                    (value, flow.default_origins, flow.used_parameters)
                };
                // Retire computation-only sets before building annotation metadata.
                // The immutable prepared catalog is shared by both resolutions.
                #[cfg(all(test, feature = "test-instrumentation"))]
                let value = if Self::test_old_annotation_lifetime() {
                    value
                } else {
                    self.typed_parameter(path, parameter, index, value)
                };
                #[cfg(not(all(test, feature = "test-instrumentation")))]
                let value = self.typed_parameter(path, parameter, index, value);
                let cached = std::sync::Arc::new(CachedParameter { value, origins });
                lease.publish(
                    std::sync::Arc::clone(&cached),
                    (path.to_path_buf(), parameter.owner),
                    used_parameters,
                );
                cached
            }
        };
        self.extend_default_origins(cached.origins.as_ref());
        cached.value.clone()
    }
}
