use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn imported_stage_array(
        &mut self,
        path: &std::path::Path,
        file: &CallableFileIndex,
        name: &str,
    ) -> Option<FlowValue> {
        let binding = file.imported.get(name)?;
        let target = self.resolver.resolve(&binding.specifier, path)?;
        let indexes = self
            .cache
            .stage_indexes
            .get_or_init(|| self.indexes.for_value_imports());
        let resolved_export = resolve_exported_callable(
            self.inputs,
            self.facts,
            self.resolver,
            &target,
            &binding.imported,
            indexes,
            &mut ExportResolutionTrace::default(),
        );
        let (target, local) = stage_export(resolved_export)?;
        let exporter = self.indexes.file(self.facts, &target)?;
        let binding = scoped_name(&exporter.values.bindings, 0, &local)?;
        if !binding.immutable
            || binding.owner.is_some()
            || binding.invalidated_at.is_some()
            || !binding.mutation_contexts.is_empty()
            || !matches!(binding.value, ValueExpression::Array(_))
            || escaped(&exporter, 0, &local)
        {
            return None;
        }
        let key = (target.clone(), local.clone());
        let slot = std::sync::Arc::clone(self.cache.stage_exports.entry(key).or_default().value());
        let closed = *slot.get_or_init(|| self.closed_stage_imports(&target, &local, indexes));
        if !closed {
            return None;
        }
        Some(self.value(&target, &binding.value))
    }

    fn closed_stage_imports(
        &self,
        target: &std::path::Path,
        local: &str,
        indexes: &CallableResolutionIndexes,
    ) -> bool {
        for path in self.inputs.graph_files.indexable() {
            let Some(file) = self.indexes.file(self.facts, path) else {
                continue;
            };
            for binding in file.imported.values() {
                let Some(resolved) = self.resolver.resolve(&binding.specifier, path) else {
                    continue;
                };
                if binding.kind
                    == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
                {
                    if self.stage_namespace_exposes(
                        &resolved,
                        target,
                        local,
                        indexes,
                        &mut fx_set(),
                    ) {
                        return false;
                    }
                    continue;
                }
                let value = resolve_exported_callable(
                    self.inputs,
                    self.facts,
                    self.resolver,
                    &resolved,
                    &binding.imported,
                    indexes,
                    &mut ExportResolutionTrace::default(),
                );
                if stage_export(value).is_some_and(|(file, name)| file == target && name == local)
                    && escaped(&file, 0, &binding.local)
                {
                    return false;
                }
            }
        }
        true
    }
}

fn escaped(file: &CallableFileIndex, scope: usize, name: &str) -> bool {
    file.values
        .for_of_escapes
        .as_ref()
        .and_then(|scopes| scopes.get(&scope))
        .is_some_and(|names| names.contains_key(name))
}

fn stage_export(value: ExportedCallableResolution) -> Option<(std::path::PathBuf, String)> {
    match value {
        ExportedCallableResolution::Value(file, name)
        | ExportedCallableResolution::Callable(file, name, _) => Some((file, name)),
        _ => None,
    }
}

impl ValueFlowResolution<'_, '_> {
    fn stage_namespace_exposes(
        &self,
        path: &std::path::Path,
        target: &std::path::Path,
        local: &str,
        indexes: &CallableResolutionIndexes,
        seen: &mut FxHashSet<std::path::PathBuf>,
    ) -> bool {
        if path == target {
            return true;
        }
        if !seen.insert(path.to_path_buf()) {
            return false;
        }
        let Some(file) = indexes.file(self.facts, path) else {
            return true;
        };
        for name in file.exported.keys() {
            let resolved = resolve_exported_callable(
                self.inputs,
                self.facts,
                self.resolver,
                path,
                name,
                indexes,
                &mut ExportResolutionTrace::default(),
            );
            if let ExportedCallableResolution::Namespace(file) = &resolved {
                if self.stage_namespace_exposes(file, target, local, indexes, seen) {
                    return true;
                }
            }
            if stage_export(resolved).is_some_and(|(file, name)| file == target && name == local) {
                return true;
            }
        }
        file.stars.iter().any(|specifier| {
            self.resolver.resolve(specifier, path).is_none_or(|file| {
                self.stage_namespace_exposes(&file, target, local, indexes, seen)
            })
        })
    }
}
