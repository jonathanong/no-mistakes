//! An index of declared module names, never a separate relationship graph.
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::TsConfigCatalog;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct AmbientModules {
    classes:
        FxHashMap<Option<usize>, FxHashMap<String, FxHashMap<String, Vec<(PathBuf, CallableId)>>>>,
}
impl AmbientModules {
    pub fn new(
        paths: &[PathBuf],
        facts: &dyn TsFactLookup,
        catalog: Option<&TsConfigCatalog>,
    ) -> Self {
        let mut result = Self::default();
        for path in paths {
            let Some(file) = facts.get_ts_facts(path) else {
                continue;
            };
            if file.parse_error.is_some() || file.operational_error.is_some() {
                continue;
            }
            if file.ancestry.ambient_modules.is_empty() {
                continue;
            }
            let classes: FxHashMap<_, _> = file
                .ancestry
                .classes
                .iter()
                .map(|class| (class.id, class))
                .collect();
            let owner = catalog.and_then(|catalog| catalog.resolver_scope_index_for(path));
            for (name, ids) in &file.ancestry.ambient_modules {
                let exports = result
                    .classes
                    .entry(owner)
                    .or_default()
                    .entry(name.clone())
                    .or_default();
                for class in ids.iter().filter_map(|id| classes.get(id)) {
                    exports
                        .entry(class.name.clone())
                        .or_default()
                        .push((path.clone(), class.id));
                }
            }
        }
        result
    }

    pub fn class(
        &self,
        specifier: &str,
        importer: &Path,
        exported: &str,
        catalog: Option<&TsConfigCatalog>,
    ) -> Option<(PathBuf, CallableId)> {
        let owner = catalog.and_then(|catalog| catalog.resolver_scope_index_for(importer));
        let candidates = self.classes.get(&owner)?.get(specifier)?.get(exported)?;
        (candidates.len() == 1).then(|| candidates[0].clone())
    }
}
