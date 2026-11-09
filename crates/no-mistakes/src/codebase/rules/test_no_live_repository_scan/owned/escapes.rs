use super::path_provenance;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct Escapes {
    local: FxHashMap<PathBuf, FxHashSet<String>>,
    unknown: bool,
}

impl Escapes {
    pub(super) fn new(
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        reachable_files: &FxHashSet<PathBuf>,
    ) -> Self {
        let mut escapes = Self::default();
        for file in graph_files
            .iter()
            .filter(|file| reachable_files.contains(*file))
        {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            match path_provenance(ts) {
                Some(paths) => {
                    escapes.unknown |= paths.modules_unknown;
                    escapes.local.insert(file.clone(), paths.value_refs.clone());
                }
                None => escapes.unknown |= ts.module_bindings.is_some(),
            }
        }
        escapes
    }

    pub(super) fn unknown(&self) -> bool {
        self.unknown
    }

    pub(super) fn contains(&self, file: &Path, name: &str) -> bool {
        let short = name.rsplit('.').next().unwrap_or(name);
        self.local
            .get(file)
            .is_some_and(|names| names.contains(name) || names.contains(short))
    }
}
