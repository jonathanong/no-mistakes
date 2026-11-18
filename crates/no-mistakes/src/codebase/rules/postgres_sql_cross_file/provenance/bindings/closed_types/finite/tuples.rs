use super::*;
use crate::codebase::dependencies::extract::RegistryKind;
use crate::fx::{fx_map, fx_set};

impl Bindings<'_> {
    pub(super) fn finite_tuple(&self, file: &Path, name: &str) -> bool {
        self.finite_tuples
            .get_or_init(|| {
                let mut files = fx_map();
                for file in self.facts.graph_file_universe() {
                    let Some(ts) = self.facts.get_ts_facts(file) else {
                        continue;
                    };
                    let flow = &ts.value_flow;
                    let mutated = flow
                        .registry_mutations
                        .iter()
                        .map(|mutation| (mutation.scope, mutation.name.as_str()))
                        .collect::<crate::fx::FxHashSet<_>>();
                    let mut names = fx_set();
                    for registry in &flow.closed_string_registries {
                        if registry.scope != 0
                            || !registry.const_tuple
                            || registry.kind != RegistryKind::Array
                            || mutated.contains(&(0, registry.name.as_str()))
                        {
                            continue;
                        }
                        let Some(values) = self.values.get(&(file, 0, registry.name.as_str()))
                        else {
                            continue;
                        };
                        let [binding] = values.as_slice() else {
                            continue;
                        };
                        if binding.immutable
                            && binding.owner.is_none()
                            && binding.invalidated_at.is_none()
                            && binding.declared_at == registry.declared_at
                        {
                            names.insert(registry.name.as_str());
                        }
                    }
                    if !names.is_empty() {
                        files.insert(file.as_path(), names);
                    }
                }
                files
            })
            .get(file)
            .is_some_and(|names| names.contains(name))
    }
}
