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
                };
                flow.visiting
                    .insert((path.to_path_buf(), parameter.scope, parameter.name.clone()));
                flow.used_parameters
                    .insert((path.to_path_buf(), parameter.owner));
                let value = flow.compute_parameter(path, parameter);
                let cached = std::sync::Arc::new(CachedParameter {
                    value,
                    dependencies: flow.used_parameters,
                });
                lease.publish(std::sync::Arc::clone(&cached));
                cached
            }
        };
        self.used_parameters
            .extend(cached.dependencies.iter().cloned());
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
            let argument = self.invocation_argument(&caller, call, parameter.index);
            values.push(match argument {
                None | Some(FlowValue::Undefined) => self.parameter_default(path, parameter),
                Some(value) => value,
            });
        }
        if values.is_empty() {
            values.push(self.parameter_default(path, parameter));
        }
        agreed_value(values)
    }
    fn parameter_default(
        &mut self,
        path: &std::path::Path,
        parameter: &ValueParameter,
    ) -> FlowValue {
        self.initializing
            .push((path.to_path_buf(), parameter.owner, parameter.index));
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
        let mut callee = self.value(caller, &call.callee);
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
