use super::*;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(in super::super) struct StageImportOrigins {
    origins: FxHashMap<PathBuf, FxHashSet<String>>,
    namespaces: FxHashSet<PathBuf>,
    unknown_namespace: bool,
}

impl StageImportOrigins {
    pub(super) fn closed(&self, target: &Path, local: &str) -> bool {
        !self.unknown_namespace
            && !self.namespaces.contains(target)
            && !self
                .origins
                .get(target)
                .is_some_and(|names| names.contains(local))
    }

    fn expose(&mut self, value: ExportedCallableResolution) {
        if let Some((file, name)) = stage_export(value) {
            self.origins.entry(file).or_default().insert(name);
        }
    }
}

impl ValueFlowResolution<'_, '_> {
    pub(super) fn stage_import_origins(
        &self,
        indexes: &CallableResolutionIndexes,
    ) -> StageImportOrigins {
        #[cfg(feature = "test-instrumentation")]
        self.cache
            .stage_import_origin_builds
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut origins = StageImportOrigins::default();
        let mut namespaces = Vec::new();
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
                    namespaces.push(resolved);
                } else if escaped(&file, 0, &binding.local) {
                    origins.expose(resolve_exported_callable(
                        self.inputs,
                        self.facts,
                        self.resolver,
                        &resolved,
                        &binding.imported,
                        indexes,
                        &mut ExportResolutionTrace::default(),
                    ));
                }
            }
        }
        while let Some(path) = namespaces.pop() {
            if !origins.namespaces.insert(path.clone()) {
                continue;
            }
            let Some(file) = indexes.file(self.facts, &path) else {
                origins.unknown_namespace = true;
                continue;
            };
            for name in file.exported.keys() {
                let resolved = resolve_exported_callable(
                    self.inputs,
                    self.facts,
                    self.resolver,
                    &path,
                    name,
                    indexes,
                    &mut ExportResolutionTrace::default(),
                );
                if let ExportedCallableResolution::Namespace(file) = resolved {
                    namespaces.push(file);
                } else {
                    origins.expose(resolved);
                }
            }
            for specifier in &file.stars {
                if let Some(file) = self.resolver.resolve(specifier, &path) {
                    namespaces.push(file);
                } else {
                    origins.unknown_namespace = true;
                }
            }
        }
        origins
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "stage_import_origins_tests.rs"]
mod tests;
