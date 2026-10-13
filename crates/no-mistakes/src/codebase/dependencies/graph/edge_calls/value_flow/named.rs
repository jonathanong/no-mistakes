use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn named(
        &mut self,
        path: &std::path::Path,
        name: &str,
        scope: Option<usize>,
        offset: u32,
    ) -> FlowValue {
        let Some(file) = self.indexes.file(self.facts, path) else {
            return FlowValue::Unknown;
        };
        let (base, member) = name
            .split_once('.')
            .map_or((name, None), |(base, member)| (base, Some(member)));
        let Some(scope) = scope.or_else(|| file.imported.contains_key(base).then_some(0)) else {
            return FlowValue::Unknown;
        };
        let key = (path.to_path_buf(), scope, base.to_string());
        if !self.visiting.insert(key.clone()) {
            return FlowValue::Unknown;
        }
        let value = if let Some(binding) = scoped_name(&file.values.bindings, scope, base) {
            if offset < binding.declared_at
                || binding
                    .invalidated_at
                    .is_some_and(|cutoff| offset >= cutoff)
            {
                FlowValue::Unknown
            } else {
                self.value(path, &binding.value)
            }
        } else if let Some(parameter) = scoped_name(&file.values.parameters, scope, base) {
            if parameter
                .invalidated_at
                .is_some_and(|cutoff| offset >= cutoff)
            {
                FlowValue::Unknown
            } else {
                self.parameter(path, parameter)
            }
        } else if let Some(id) = file.resolve_local_callable_id(Some(scope), base) {
            self.callable(path, id)
        } else if scope == 0 {
            let Some(binding) = file.imported.get(base) else {
                self.visiting.remove(&key);
                return FlowValue::Unknown;
            };
            let imported_value = self
                .resolver
                .resolve(&binding.specifier, path)
                .and_then(|target| {
                    self.inputs
                        .graph_files
                        .visible_path(&target)
                        .map(std::path::Path::to_path_buf)
                })
                .map(|target| {
                    resolve_exported_callable(
                        self.inputs,
                        self.facts,
                        self.resolver,
                        &target,
                        &binding.imported,
                        self.indexes,
                        &mut Vec::new(),
                    )
                });
            match imported_value {
                Some(ExportedCallableResolution::Value(target, local)) => {
                    let value = self.named(&target, &local, Some(0), u32::MAX);
                    match value {
                        FlowValue::Callable {
                            file: target,
                            scope,
                            id,
                        } => FlowValue::Target(
                            module_export_target(&file, base, Some((target, scope)), Some(id))
                                .expect("import binding"),
                        ),
                        FlowValue::Returned { factory, executor, .. } => {
                            let receiver = match self.imported_value(&file, base) { FlowValue::Target(target) => Some(target), _ => None };
                            FlowValue::Returned { factory, executor, receiver }
                        },
                        _ => self.imported_value(&file, base),
                    }
                }
                Some(ExportedCallableResolution::Callable(target, scope, id)) => FlowValue::Target(module_export_target(&file, base, Some((target, scope)), id).expect("import binding")),
                Some(ExportedCallableResolution::ExternalModuleExport(specifier, export_path)) => FlowValue::Target(ResolvedCallTarget::ModuleExport { specifier, export_path, repository_target: None, callable_id: None }),
                Some(ExportedCallableResolution::Unknown | ExportedCallableResolution::Absent) if binding.kind != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace => FlowValue::Unknown,
                _ => self.imported_value(&file, base),
            }
        } else {
            FlowValue::Unknown
        };
        self.visiting.remove(&key);
        member.map_or(value.clone(), |member| self.member(value, member))
    }
    fn imported_value(&self, file: &CallableFileIndex, base: &str) -> FlowValue {
        if let Some(binding) = file.imported.get(base).filter(|binding| {
            binding.kind == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
        }) {
            FlowValue::Target(ResolvedCallTarget::ModuleExport {
                specifier: binding.specifier.clone(),
                export_path: String::new(),
                repository_target: None,
                callable_id: None,
            })
        } else {
            FlowValue::Target(
                module_export_target(file, base, None, None).unwrap_or(ResolvedCallTarget::Unknown),
            )
        }
    }
    pub(super) fn parameter(
        &mut self,
        path: &std::path::Path,
        parameter: &ValueParameter,
    ) -> FlowValue {
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
            if call
                .spread_at
                .is_some_and(|spread| parameter.index >= spread)
            {
                values.push(FlowValue::Unknown);
                continue;
            }
            let argument = call.arguments.get(parameter.index);
            let omitted = argument.is_none() || argument.is_some_and(|value| matches!(value, ValueExpression::Named { name, scope: None, .. } if name == "undefined"));
            values.push(if omitted {
                self.value(path, &parameter.default)
            } else {
                self.value(&caller, argument.expect("present argument"))
            });
        }
        if values.is_empty() {
            values.push(self.value(path, &parameter.default));
        }
        let value = agreed_value(values);
        self.parameters.insert(key, value.clone());
        value
    }
}
