use super::{Environment, Evaluator};
use crate::codebase::postgres::query_annotation::module_ids::{ModuleId, ModuleIds};
use crate::fx::FxHashMap;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// Snapshots copy compact keys and share the request's immutable identity table.
#[derive(Clone, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct ModuleCache {
    ids: OnceLock<Arc<ModuleIds>>,
    values: Arc<FxHashMap<ModuleId, Environment>>,
}

impl ModuleCache {
    pub fn with_ids(ids: Arc<ModuleIds>) -> Self {
        Self {
            ids: OnceLock::from(ids),
            values: Arc::default(),
        }
    }
}

impl Deref for ModuleCache {
    type Target = FxHashMap<ModuleId, Environment>;
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl DerefMut for ModuleCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.values)
    }
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn module_ids(&self) -> &ModuleIds {
        self.shared_module_ids()
    }

    pub(in crate::codebase::postgres::query_annotation) fn shared_module_ids(
        &self,
    ) -> &Arc<ModuleIds> {
        self.modules.ids.get_or_init(|| {
            // Production evaluators already have the prepared shared table.
            // Direct unit evaluators freeze their supplied catalog lazily.
            // Every prepared module carries the same Arc, so the chosen table
            // is independent of hash-map iteration order.
            self.files
                .values()
                .find_map(|file| file.module.module_ids.as_ref())
                .map(Arc::clone)
                .unwrap_or_else(|| Arc::new(ModuleIds::new(self.files.keys().cloned())))
        })
    }
}

#[cfg(test)]
mod tests;
