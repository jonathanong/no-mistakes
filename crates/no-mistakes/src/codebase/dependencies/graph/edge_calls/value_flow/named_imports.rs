use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn imported_named_value(
        &mut self,
        path: &std::path::Path,
        file: &CallableFileIndex,
        base: &str,
        binding: &crate::codebase::dependencies::extract::ImportedBinding,
    ) -> FlowValue {
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
                        module_export_target(file, base, Some((target, scope)), Some(id))
                            .expect("import binding"),
                    ),
                    FlowValue::Returned {
                        factory, executor, ..
                    } => {
                        let receiver = match self.imported_value(file, base) {
                            FlowValue::Target(target) => Some(target),
                            _ => None,
                        };
                        FlowValue::Returned {
                            factory,
                            executor,
                            receiver,
                        }
                    }
                    value @ (FlowValue::Promise(_)
                    | FlowValue::Bound { .. }
                    | FlowValue::Uncallable) => value,
                    _ => self.imported_value(file, base),
                })
            }
            Some(ExportedCallableResolution::Callable(target, scope, id)) => FlowValue::Target(
                module_export_target(file, base, Some((target, scope)), id)
                    .expect("import binding"),
            ),
            Some(ExportedCallableResolution::ExternalModuleExport(specifier, export_path)) => {
                FlowValue::Target(ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    repository_target: None,
                    callable_id: None,
                })
            }
            Some(ExportedCallableResolution::Unknown | ExportedCallableResolution::Absent)
                if binding.kind
                    != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace =>
            {
                FlowValue::Unknown
            }
            _ => self.imported_value(file, base),
        }
    }
}
