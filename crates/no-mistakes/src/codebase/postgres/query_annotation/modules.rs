use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Immutable syntax indexes and lazy targets for one resolver/catalog projection.
/// Executor profiles share this data, but never share mutable evaluator state.
pub(crate) struct Module {
    pub ts: Arc<TsFileFacts>,
    pub imports: FxHashMap<String, usize>,
    pub exports: FxHashMap<String, usize>,
    targets: FxHashMap<String, OnceLock<Option<PathBuf>>>,
}

impl Module {
    pub fn new(ts: Arc<TsFileFacts>) -> Self {
        let mut imports = crate::fx::fx_map();
        let mut exports = crate::fx::fx_map();
        let mut targets = crate::fx::fx_map();
        for (index, binding) in ts.imported_bindings.iter().enumerate() {
            if !binding.is_type_only {
                imports.entry(binding.local.clone()).or_insert(index);
                targets
                    .entry(binding.specifier.clone())
                    .or_insert_with(OnceLock::new);
            }
        }
        // Demand considers all runtime dependencies, even when evaluation never
        // reads an export (including bare imports and function-local require).
        for import in &ts.imports {
            if import.kind != ImportKind::Type && !import.computed {
                targets
                    .entry(import.specifier.clone())
                    .or_insert_with(OnceLock::new);
            }
        }
        for specifier in &ts.eager_require_specifiers {
            targets
                .entry(specifier.clone())
                .or_insert_with(OnceLock::new);
        }
        for (index, binding) in ts.exported_bindings.iter().enumerate() {
            exports.entry(binding.exported.clone()).or_insert(index);
            if let Some(specifier) = &binding.specifier {
                targets
                    .entry(specifier.clone())
                    .or_insert_with(OnceLock::new);
            }
        }
        for specifier in &ts.star_reexport_specifiers {
            targets
                .entry(specifier.clone())
                .or_insert_with(OnceLock::new);
        }
        crate::diagnostics::with_current(|observer| {
            if let Some(observer) = observer {
                observer.increment("postgres.annotation.modules", 1);
            }
        });
        Self {
            ts,
            imports,
            exports,
            targets,
        }
    }

    pub fn target(
        &self,
        specifier: &str,
        from: &Path,
        resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
    ) -> Option<&Path> {
        self.targets[specifier]
            .get_or_init(|| {
                crate::diagnostics::with_current(|observer| {
                    if let Some(observer) = observer {
                        observer.increment("postgres.annotation.target_resolutions", 1);
                    }
                });
                resolve(specifier, from)
            })
            .as_deref()
    }
}

#[cfg(test)]
mod tests;
