use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn compute_parameter(
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
        let synchronous = self.synchronous_callable(path, parameter.owner);
        let has_concrete = synchronous
            && incoming.iter().any(|input| {
                if input.callback {
                    return false;
                }
                let call = self
                    .indexes
                    .file(self.facts, &input.file)
                    .and_then(|index| {
                        index
                            .values
                            .sites
                            .get(&input.offset)
                            .and_then(|sites| sites.get(&input.callee))
                            .and_then(|key| index.values.calls.get(key).cloned())
                    });
                call.is_some_and(|call| self.safe_inline_callee(&input.file, &call.callee))
            });
        let unique = incoming.len() == 1;
        for input in incoming {
            let ParameterInput {
                file: caller,
                offset,
                callee,
                synthetic_callback,
                callback,
            } = input;
            if callback {
                if !(synthetic_callback && has_concrete) {
                    values.push(FlowValue::Unknown);
                }
                continue;
            }
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
            if super::inline_inputs::inline_callee(&call.callee)
                && (!synchronous || !self.safe_inline_callee(&caller, &call.callee))
            {
                values.push(FlowValue::Unknown);
                continue;
            }
            self.default_origins = None;
            let previous_foreign = self.foreign_acquisition;
            let previous_read = self.invocation_read.clone();
            let previous_start = self.invocation_start;
            let previous_execution = self.executing_read;
            self.enter_inline_read(&caller, call, path, parameter.owner);
            if synchronous {
                self.enter_parameter_read(&caller, call, path, parameter.owner);
            }
            self.foreign_acquisition =
                self.parameter_acquisition_repeats(&caller, call, path, parameter.owner, unique);
            let argument = self.invocation_argument(&caller, call, parameter.index);
            let from_default = argument.as_ref().is_none_or(FlowValue::is_undefined);
            let acquisition = self
                .indexes
                .file(self.facts, path)
                .filter(|index| index.values.receiver_mutations.is_some())
                .filter(|_| {
                    from_default
                        || parameter
                            .projection
                            .iter()
                            .any(|step| matches!(step, ValueProjection::Default(..)))
                })
                .map(|_| {
                    self.make_receiver_use(&caller, call, None, Some((path, parameter.owner, true)))
                });
            let previous = self.acquisition_context.take();
            if from_default {
                self.acquisition_context = acquisition.clone();
            }
            let value = if from_default {
                self.parameter_default(path, parameter)
            } else {
                argument.expect("supplied non-undefined argument")
            };
            if !parameter.projection.is_empty() {
                // Destructuring acquires methods before the body executes. Carry
                // caller timing into lazy receiver fields before they snapshot.
                self.record_receiver_use(
                    &caller,
                    call,
                    &value,
                    from_default.then_some((path, parameter.owner, true)),
                );
            }
            values.push(self.project_parameter(path, parameter, value, from_default, acquisition));
            if self.acquisition_context.is_some() {
                self.seal_receiver_acquisition();
            }
            self.acquisition_context = previous;
            self.record_receiver_use(
                &caller,
                call,
                values.last().expect("projected argument"),
                from_default.then_some((path, parameter.owner, true)),
            );
            self.invocation_read = previous_read;
            self.invocation_start = previous_start;
            self.executing_read = previous_execution;
            self.foreign_acquisition = previous_foreign;
            default_origins::merge(&mut origins, self.default_origins.take(), self.cache);
        }
        if values.is_empty() {
            let value = self.parameter_default(path, parameter);
            values.push(self.project_parameter(path, parameter, value, true, None));
        }
        default_origins::merge(&mut origins, self.default_origins.take(), self.cache);
        self.default_origins = origins;
        agreed_value(values)
    }
}
