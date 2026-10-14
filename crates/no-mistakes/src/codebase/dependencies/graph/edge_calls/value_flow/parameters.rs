use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn parameter(
        &mut self,
        path: &std::path::Path,
        parameter: &ValueParameter,
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
                    cache: self.cache,
                    parameters_enabled: true,
                    initializing: self.initializing.clone(),
                    used_parameters: fx_set(),
                    visiting: fx_set(),
                    calls_visiting: fx_set(),
                    default_origins: None,
                };
                flow.visiting
                    .insert((path.to_path_buf(), parameter.scope, parameter.name.clone()));
                flow.used_parameters
                    .insert((path.to_path_buf(), parameter.owner));
                let value = flow.compute_parameter(path, parameter);
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
        cached.value.clone()
    }
    fn compute_parameter(
        &mut self,
        path: &std::path::Path,
        parameter: &ValueParameter,
    ) -> FlowValue {
        let incoming = self
            .incoming
            .get(&(path.to_path_buf(), parameter.owner))
            .cloned()
            .unwrap_or_default();
        let mut values = Vec::new();
        let mut origins = None;
        for (caller, offset, callee) in incoming {
            let Some(index) = self.indexes.file(self.facts, &caller) else {
                continue;
            };
            let call = index
                .values
                .sites
                .get(&offset)
                .and_then(|sites| sites.get(callee.as_str()))
                .and_then(|key| index.values.calls.get(key));
            let Some(call) = call else {
                values.push(FlowValue::Unknown);
                continue;
            };
            self.default_origins = None;
            let argument = self.invocation_argument(&caller, call, parameter.index);
            let from_default = matches!(argument, None | Some(FlowValue::Undefined));
            let value = match argument {
                None | Some(FlowValue::Undefined) => self.parameter_default(path, parameter),
                Some(value) => value,
            };
            values.push(self.project_parameter(path, parameter, value, from_default));
            default_origins::merge(&mut origins, self.default_origins.take(), self.cache);
        }
        if values.is_empty() {
            let value = self.parameter_default(path, parameter);
            values.push(self.project_parameter(path, parameter, value, true));
        }
        default_origins::merge(&mut origins, self.default_origins.take(), self.cache);
        self.default_origins = origins;
        agreed_value(values)
    }
    fn parameter_default(
        &mut self,
        path: &std::path::Path,
        parameter: &ValueParameter,
    ) -> FlowValue {
        self.initializing
            .push((path.to_path_buf(), parameter.owner, parameter.default_order));
        let value = self.value(path, &parameter.default);
        self.initializing.pop();
        value
    }
    fn invocation_argument(
        &mut self,
        caller: &std::path::Path,
        call: &ValueCall,
        mut index: usize,
    ) -> Option<FlowValue> {
        let origins = self.default_origins.take();
        let mut callee = self.value(caller, &call.callee);
        self.default_origins = origins;
        while let FlowValue::Returned { executor, .. } = callee {
            callee = *executor;
        }
        if let FlowValue::Bound { arguments, .. } = callee {
            for bound in arguments {
                if bound.spread_at.is_some_and(|spread| index >= spread) {
                    return Some(FlowValue::Unknown);
                }
                if let Some(argument) = bound.arguments.get(index) {
                    return Some(self.value(&bound.file, argument));
                }
                index -= bound.arguments.len();
            }
        }
        if call.spread_at.is_some_and(|spread| index >= spread) {
            return Some(FlowValue::Unknown);
        }
        call.arguments
            .get(index)
            .map(|argument| self.value(caller, argument))
    }
}
