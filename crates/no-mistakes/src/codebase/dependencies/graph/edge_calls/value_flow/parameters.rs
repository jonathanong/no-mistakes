use super::*;

#[path = "parameter_agreement.rs"]
mod agreement;

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
                let mut flow = ValueFlowResolution {
                    inputs: self.inputs,
                    facts: self.facts,
                    resolver: self.resolver,
                    indexes: self.indexes,
                    incoming: self.incoming,
                    supers: self.supers,
                    replacements: self.replacements,
                    replacement_read: false,
                    // Agreement values depend only on indexed incoming sources,
                    // never on the ambient query which first fills the cache.
                    foreign_acquisition: false,
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
                flow.visiting
                    .insert((path.to_path_buf(), parameter.scope, parameter.name.clone()));
                flow.used_parameters
                    .insert((path.to_path_buf(), parameter.owner));
                let value = flow.compute_parameter(path, parameter);
                flow.merge_writer_dependencies();
                let cached = std::sync::Arc::new(CachedParameter {
                    value,
                    origins: flow.default_origins,
                });
                lease.publish(
                    std::sync::Arc::clone(&cached),
                    (path.to_path_buf(), parameter.owner),
                    flow.used_parameters,
                );
                cached
            }
        };
        self.extend_default_origins(cached.origins.as_ref());
        self.typed_parameter(path, parameter, index, cached.value.clone())
    }
}
