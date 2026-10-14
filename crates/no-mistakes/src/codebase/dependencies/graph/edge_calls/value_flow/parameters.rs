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
                    member_read: None,
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
        self.typed_parameter(path, parameter, index, cached.value.clone())
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
            self.foreign_acquisition = self
                .replacements
                .is_some_and(|catalog| catalog.possible_repeat(&caller, call.offset, call.context));
            let previous_read = self.invocation_read.clone();
            if super::inline_inputs::inline_callee(&call.callee) {
                self.invocation_read = Some(default_origins::InvocationRead {
                    file: caller.clone(),
                    offset: call.offset,
                    context: call.context,
                    owner: (path.to_path_buf(), parameter.owner),
                    once: call.once,
                });
            }
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
