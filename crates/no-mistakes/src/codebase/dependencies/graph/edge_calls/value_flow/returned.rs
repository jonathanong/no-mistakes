use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn returned(&mut self, path: &std::path::Path, offset: u32) -> FlowValue {
        let key = (path.to_path_buf(), offset);
        if !self.calls_visiting.insert(key.clone()) {
            return FlowValue::Unknown;
        }
        let mut factory = None;
        let result = self
            .indexes
            .file(self.facts, path)
            .and_then(|file| file.values.calls.get(&offset).cloned())
            .map(|call| {
                let target = self.value(path, &call.callee);
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
                let Some(returns) = index.values.returns.get(&id) else {
                    return FlowValue::Unknown;
                };
                agreed_value(
                    returns
                        .iter()
                        .map(|value| self.value(&file, value))
                        .collect(),
                )
            })
            .unwrap_or(FlowValue::Unknown);
        self.calls_visiting.remove(&key);
        match (factory, result) {
            (
                Some(origin),
                FlowValue::Returned {
                    mut factory,
                    executor,
                },
            ) => {
                factory.retain(|previous| previous != &origin);
                factory.insert(0, origin);
                FlowValue::Returned { factory, executor }
            }
            (
                Some(factory),
                executor @ (FlowValue::Callable { .. } | FlowValue::Target(_) | FlowValue::Unknown),
            ) => FlowValue::Returned {
                factory: vec![factory],
                executor: Box::new(executor),
            },
            (_, result) => result,
        }
    }
}
pub(super) fn agreed_value(values: Vec<FlowValue>) -> FlowValue {
    let Some(first) = values.first() else {
        return FlowValue::Unknown;
    };
    if values.iter().all(|value| value == first) {
        first.clone()
    } else {
        FlowValue::Unknown
    }
}
