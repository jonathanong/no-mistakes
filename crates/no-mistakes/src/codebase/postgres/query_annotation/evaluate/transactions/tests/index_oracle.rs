//! Full scans validate sparse index maintenance at real arm boundaries.
use super::super::Evaluator;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
impl<F> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn with_checkpoint_index_oracle(
        mut self,
    ) -> Self {
        self.transactions.verify_indexes = true;
        self
    }
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn assert_checkpoint_indexes(
        &self,
    ) {
        if !self.transactions.verify_indexes {
            return;
        }
        let mut readers = FxHashMap::<_, FxHashMap<_, FxHashSet<_>>>::default();
        for (frame, bindings) in &self.captured_bindings {
            for (name, origin) in bindings {
                readers
                    .entry(*origin)
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .insert(*frame);
            }
        }
        assert!(
            readers == self.captured_binding_readers,
            "captured reader index equals full rebuild"
        );
        let mut owners = FxHashMap::default();
        for (frame, mappings) in &self.mapped_arguments {
            for (id, params) in mappings {
                for name in params.iter().filter(|name| !name.is_empty()) {
                    let origin = self
                        .captured_bindings
                        .get(frame)
                        .and_then(|bindings| bindings.get(name))
                        .copied()
                        .unwrap_or(*frame);
                    owners.entry(*id).or_insert(origin);
                }
            }
        }
        assert!(
            owners == self.mapped_argument_owners,
            "mapped owner index equals full rebuild"
        );
        let mut indices = FxHashMap::<_, FxHashMap<_, _>>::default();
        for (id, owner) in &self.mapped_argument_owners {
            let params = &self.mapped_arguments[owner]
                .iter()
                .find(|(object, _)| object == id)
                .expect("canonical owner retains its map")
                .1;
            indices.insert(
                *id,
                params
                    .iter()
                    .enumerate()
                    .filter(|(_, name)| !name.is_empty())
                    .map(|(index, name)| (name.clone(), index))
                    .collect(),
            );
        }
        assert!(
            indices == self.mapped_parameter_indices,
            "mapped parameter index equals full rebuild"
        );
        let mut freshness = FxHashMap::<_, FxHashMap<_, FxHashSet<_>>>::default();
        for (frame, names) in &self.fresh_mapped_parameters {
            for (id, params) in self.mapped_arguments.get(frame).into_iter().flatten() {
                for name in names {
                    let Some(index) = indices.get(id).and_then(|indices| indices.get(name)) else {
                        continue;
                    };
                    if params.get(*index) == Some(name)
                        && !self.disconnected_argument_slots.contains(&(*id, *index))
                    {
                        freshness
                            .entry(*id)
                            .or_default()
                            .entry(*frame)
                            .or_default()
                            .insert(name.clone());
                    }
                }
            }
        }
        assert!(
            freshness == self.fresh_mapped_argument_bindings,
            "freshness index equals full rebuild"
        );
    }
}
