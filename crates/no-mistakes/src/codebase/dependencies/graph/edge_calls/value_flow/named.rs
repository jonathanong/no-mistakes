use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn named(
        &mut self,
        path: &std::path::Path,
        name: &str,
        scope: Option<usize>,
        offset: u32,
        context: Option<CallableId>,
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> FlowValue {
        let (offset, context) = self.invocation_read.as_ref()
            .filter(|(file, _, _, _)| file == path)
            .map_or((offset, context), |(_, offset, context, _)| (*offset, *context));
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
        if let Some(parameter) = scoped_name(&file.values.parameters, scope, base) {
            self.used_parameters
                .insert((path.to_path_buf(), parameter.owner));
        }
        if !self.visiting.insert(key.clone()) {
            return FlowValue::Unknown;
        }
        let value = if let Some(binding) = scoped_name(&file.values.bindings, scope, base) {
            if binding
                .mutation_contexts
                .iter()
                .any(|writer| *writer != context)
                || offset < binding.declared_at
                || binding
                    .invalidated_at
                    .is_some_and(|cutoff| offset >= cutoff)
            {
                FlowValue::Unknown
            } else {
                self.value(path, &binding.value)
            }
        } else if let Some(parameter) = scoped_name(&file.values.parameters, scope, base) {
            if self.initializing.iter().any(|(file, owner, index)| {
                file == path
                    && *owner == parameter.owner
                    && parameter.initialization_order >= *index
            }) || parameter
                .mutation_contexts
                .iter()
                .any(|writer| *writer != context)
                || parameter
                    .invalidated_at
                    .is_some_and(|cutoff| offset >= cutoff)
            {
                FlowValue::Unknown
            } else {
                let value = self.parameter(path, parameter, &file);
                if context.is_some_and(|context| {
                    file.values
                        .receiver_mutations
                        .as_ref()
                        .is_some_and(|receivers| receivers.writer_contexts.contains(&context))
                }) {
                    self.record_parameter_receiver_read(path, offset, context);
                }
                value
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
                    let value = self.named(&target, &local, Some(0), u32::MAX, None, None);
                    value.map_executor(|value| match value {
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
                        value @ (FlowValue::Promise(_) | FlowValue::Bound { .. } | FlowValue::Uncallable) => value,
                        _ => self.imported_value(&file, base),
                    })
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
        if scope == 0 && file.imported.contains_key(base) {
            self.record_receiver_origin(path, scope, base, offset, context);
        }
        if let Some((source, read_offset, read_context, owner)) = &self.invocation_read {
            if let Some(history) = file.values.receiver_mutations.as_ref().and_then(|histories| histories.get(&scope)).and_then(|names| names.get(base)) {
                let origin = self.cache.origin_node(OriginKind::InvocationReceiver {
                    file: source.clone(), offset: *read_offset, context: *read_context, owner: *owner,
                    history: std::sync::Arc::clone(history),
                }, 1);
                default_origins::merge(&mut self.default_origins, Some(origin), self.cache);
            }
        }
        let value = if self.default_origin_invalidated(path, offset, context, projection) {
            FlowValue::Unknown
        } else {
            value
        };
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
}
