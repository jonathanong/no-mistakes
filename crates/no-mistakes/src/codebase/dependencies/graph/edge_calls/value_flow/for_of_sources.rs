use super::*;

#[path = "stage_import_origins.rs"]
mod stage_import_origins;
pub(super) use stage_import_origins::StageImportOrigins;

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
        let origins = self
            .cache
            .stage_import_origins
            .get_or_init(|| self.stage_import_origins(indexes));
        if !origins.closed(&target, &local) {
            return None;
        }
        Some(self.value(&target, &binding.value))
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
