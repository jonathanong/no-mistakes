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
        let key = (path.to_path_buf(), parameter.scope, parameter.name.clone());
        if let Some(value) = self.parameters.get(&key) {
            return value.clone();
        }
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
                None | Some(FlowValue::Undefined) => self.value(path, &parameter.default),
                Some(value) => value,
            });
        }
        if values.is_empty() {
            values.push(self.value(path, &parameter.default));
        }
        let value = agreed_value(values);
        self.parameters.insert(key, value.clone());
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
