use super::Bindings;
use crate::codebase::dependencies::extract::ValueParameter;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::Path;

pub(super) type ParameterIndex<'a> = FxHashMap<&'a Path, FileParameters<'a>>;

#[derive(Default)]
pub(super) struct FileParameters<'a> {
    scopes: FxHashMap<usize, FxHashMap<&'a str, Vec<&'a ValueParameter>>>,
    names: FxHashSet<&'a str>,
}

impl<'a> Bindings<'a> {
    pub(super) fn scoped_parameters(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
    ) -> &[&'a ValueParameter] {
        self.file_parameters(file)
            .and_then(|parameters| parameters.scopes.get(&scope))
            .and_then(|names| names.get(name))
            .map_or(&[], Vec::as_slice)
    }

    pub(super) fn has_parameter_name(&self, file: &Path, name: &str) -> bool {
        self.file_parameters(file)
            .is_some_and(|parameters| parameters.names.contains(name))
    }

    fn file_parameters(&self, file: &Path) -> Option<&FileParameters<'a>> {
        let index = self.parameter_index();
        index.get(file).or_else(|| {
            let normalized = crate::codebase::ts_resolver::normalize_path(file);
            index.get(normalized.as_path())
        })
    }

    fn parameter_index(&self) -> &ParameterIndex<'a> {
        self.parameters.get_or_init(|| {
            let mut index = ParameterIndex::default();
            for (file, facts) in &self.facts.ts {
                let entry = index.entry(file).or_default();
                for parameter in &facts.ts.value_flow.parameters {
                    entry.names.insert(&parameter.name);
                    entry
                        .scopes
                        .entry(parameter.scope)
                        .or_default()
                        .entry(&parameter.name)
                        .or_default()
                        .push(parameter);
                }
            }
            index
        })
    }
}
