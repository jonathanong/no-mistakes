use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn returned(&mut self, path: &std::path::Path, offset: u32, end: u32) -> FlowValue {
        let key = (path.to_path_buf(), offset, end);
        if !self.calls_visiting.insert(key.clone()) {
            return FlowValue::Unknown;
        }
        let mut factory = None;
        let mut is_async = false;
        let result = self
            .indexes
            .file(self.facts, path)
            .and_then(|file| file.values.calls.get(&(offset, end)).cloned())
            .map(|call| {
                let target = self.value(path, &call.callee);
                if call.bound {
                    return target.map_executor(|target| {
                        let (target, mut arguments) = match target {
                            FlowValue::Bound { target, arguments } => (target, arguments),
                            target => (Box::new(target), Vec::new()),
                        };
                        arguments.push(BoundArguments {
                            file: path.to_path_buf(),
                            arguments: std::sync::Arc::clone(&call.arguments),
                            spread_at: call.spread_at,
                        });
                        FlowValue::Bound { target, arguments }
                    });
                }
                let mut target = target;
                loop {
                    target = match target {
                        FlowValue::Bound { target, .. }
                        | FlowValue::Provenance {
                            executor: target, ..
                        } => *target,
                        value => {
                            target = value;
                            break;
                        }
                    };
                }

                if let FlowValue::Target(ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    ..
                }) = &target
                {
                    factory = Some((specifier.clone(), export_path.clone()));
                }
                let target = match target {
                    FlowValue::Callable { file, id, .. } => Some((file, id)),
                    FlowValue::Target(ResolvedCallTarget::ModuleExport {
                        repository_target: Some((file, _)),
                        callable_id: Some(id),
                        ..
                    }) => Some((file, id)),
                    FlowValue::Target(ResolvedCallTarget::ModuleExport {
                        specifier,
                        export_path,
                        ..
                    }) => self
                        .resolver
                        .resolve(&specifier, path)
                        .and_then(|file| {
                            self.inputs
                                .graph_files
                                .visible_path(&file)
                                .map(std::path::Path::to_path_buf)
                        })
                        .and_then(|file| {
                            match resolve_exported_callable(
                                self.inputs,
                                self.facts,
                                self.resolver,
                                &file,
                                &export_path,
                                self.indexes,
                                &mut Vec::new(),
                            ) {
                                ExportedCallableResolution::Callable(file, _, Some(id)) => {
                                    Some((file, id))
                                }
                                _ => None,
                            }
                        }),
                    _ => None,
                };
                let Some((file, id)) = target else {
                    return FlowValue::Unknown;
                };
                let Some(index) = self.indexes.file(self.facts, &file) else {
                    return FlowValue::Unknown;
                };
                if index.values.generator_callables.contains(&id) {
                    return FlowValue::Uncallable;
                }
                is_async = index.values.async_callables.contains(&id);
                let Some(returns) = index.values.returns.get(&id) else {
                    return FlowValue::Unknown;
                };
                let previous_execution = self.executing_read;
                if !is_async {
                    self.enter_returned_read(path, &call, &file, id);
                }
                let previous = self.acquisition_context.take();
                self.acquisition_context = index.values.receiver_mutations.as_ref().map(|_| {
                    self.make_receiver_use(path, &call, previous.clone(), Some((&file, id, false)))
                });
                let value = agreed_value(
                    returns
                        .iter()
                        .map(|value| self.value(&file, value))
                        .collect(),
                );
                if self.acquisition_context.is_some() {
                    self.seal_receiver_acquisition();
                }
                self.acquisition_context = previous;
                self.executing_read = previous_execution;
                self.record_receiver_use(path, &call, &value, Some((&file, id, false)));
                value
            })
            .unwrap_or(FlowValue::Unknown);
        self.calls_visiting.remove(&key);
        let mut result = result;
        if is_async {
            result = result.awaited();
        }
        let result = with_factory_origin(factory, result);
        if is_async {
            FlowValue::Promise(Box::new(result))
        } else {
            result
        }
    }
}

fn with_factory_origin(factory: Option<(String, String)>, result: FlowValue) -> FlowValue {
    result.map_executor(|result| match (factory, result) {
        (Some(origin), FlowValue::Promise(inner)) => {
            FlowValue::Promise(Box::new(with_factory_origin(Some(origin), *inner)))
        }
        (
            Some(origin),
            FlowValue::Returned {
                mut factory,
                executor,
                receiver,
            },
        ) => {
            factory.retain(|previous| previous != &origin);
            factory.insert(0, origin);
            FlowValue::Returned {
                factory,
                executor,
                receiver,
            }
        }
        (
            Some(factory),
            executor @ (FlowValue::Callable { .. }
            | FlowValue::Bound { .. }
            | FlowValue::Target(_)
            | FlowValue::Unknown),
        ) => FlowValue::Returned {
            factory: vec![factory],
            executor: Box::new(executor),
            receiver: None,
        },
        (Some(factory), executor @ (FlowValue::Object { .. } | FlowValue::Array { .. })) => {
            FlowValue::Provenance {
                executor: Box::new(executor),
                typed: Vec::new(),
                members: Vec::new(),
                receiver_factories: vec![factory],
            }
        }
        (_, result) => result,
    })
}
