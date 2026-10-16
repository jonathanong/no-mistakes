use super::provenance::type_export;
use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn annotate(executor: FlowValue, annotation: FlowValue) -> FlowValue {
        match annotation {
            FlowValue::Provenance {
                typed,
                executor: inner,
                members,
                receiver_factories,
            } => FlowValue::Provenance {
                executor: Box::new(Self::annotate(executor, *inner)),
                typed,
                members,
                receiver_factories,
            },
            FlowValue::TypedObject { properties, .. } => FlowValue::TypedObject {
                executor: Box::new(executor),
                properties,
            },
            _ => executor,
        }
    }

    pub(super) fn annotation_value(
        &self,
        path: &std::path::Path,
        shape: &ValueType,
        module: Option<&str>,
        visited: &mut FxHashSet<(std::path::PathBuf, String)>,
    ) -> FlowValue {
        match shape {
            ValueType::Unknown => FlowValue::Unknown,
            ValueType::Object(properties) => FlowValue::TypedObject {
                executor: Box::new(FlowValue::Unknown),
                properties: properties
                    .iter()
                    .map(|(name, shape)| {
                        (
                            name.clone(),
                            self.annotation_value(path, shape, module, visited),
                        )
                    })
                    .collect(),
            },
            ValueType::Member(base, member) => {
                let mut value = self.annotation_value(path, base, module, visited);
                while let FlowValue::Provenance { executor, .. } = value {
                    value = *executor;
                }
                match value {
                    FlowValue::TypedObject { mut properties, .. } => {
                        properties.remove(member).unwrap_or(FlowValue::Unknown)
                    }
                    _ => FlowValue::Unknown,
                }
            }
            ValueType::Reference(reference) => {
                let key = (path.to_path_buf(), reference.clone());
                if !visited.insert(key.clone()) {
                    return FlowValue::Unknown;
                }
                let value = self.annotation_reference(path, reference, module, visited);
                visited.remove(&key);
                value
            }
        }
    }

    fn annotation_reference(
        &self,
        path: &std::path::Path,
        reference: &str,
        module: Option<&str>,
        visited: &mut FxHashSet<(std::path::PathBuf, String)>,
    ) -> FlowValue {
        let Some(index) = self.indexes.file(self.facts, path) else {
            return FlowValue::Unknown;
        };
        let (local, member) = reference
            .split_once('.')
            .map_or((reference, None), |(a, b)| (a, Some(b)));
        if let Some(binding) = index.type_imported.get(local) {
            let Some(export) = type_export(binding, member) else {
                return FlowValue::Unknown;
            };
            let origin = self
                .cache
                .type_origin((path.to_path_buf(), reference.to_owned()), || {
                    self.type_origin(path, binding.specifier.clone(), export.clone())
                });
            let inner = self
                .resolver
                .resolve(&binding.specifier, path)
                .and_then(|target| {
                    self.inputs
                        .graph_files
                        .visible_path(&target)
                        .map(std::path::Path::to_path_buf)
                })
                .map_or(FlowValue::Unknown, |target| {
                    self.annotation_value(
                        &target,
                        &ValueType::Reference(export),
                        Some(&origin.0),
                        visited,
                    )
                });
            return FlowValue::Provenance {
                executor: Box::new(inner),
                typed: vec![origin],
                members: Vec::new(),
                receiver_factories: Vec::new(),
            };
        }
        let Some(facts) = self.facts.get_ts_facts(path) else {
            return FlowValue::Unknown;
        };
        let inner = facts
            .value_flow
            .type_shapes
            .get(reference)
            .map_or(FlowValue::Unknown, |shape| {
                self.annotation_value(path, shape, module, visited)
            });
        if let Some(module) = module {
            let typed: Vec<_> = facts
                .value_flow
                .type_exports
                .iter()
                .filter(|binding| binding.specifier.is_none() && binding.local == reference)
                .map(|binding| (module.to_owned(), binding.exported.clone()))
                .collect();
            if !typed.is_empty() {
                return FlowValue::Provenance {
                    executor: Box::new(inner),
                    typed,
                    members: Vec::new(),
                    receiver_factories: Vec::new(),
                };
            }
        }
        inner
    }
}
