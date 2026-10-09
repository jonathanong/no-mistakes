use super::{Environment, Evaluator};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn mapped_slot_target(
        &self,
        id: u64,
        index: usize,
    ) -> Option<(Environment, String)> {
        let owner = *self.mapped_argument_owners.get(&id)?;
        // Registration and metadata compaction preserve the owner's own map.
        let params = &self.mapped_arguments[&owner]
            .iter()
            .find(|(object, _)| *object == id)
            .expect("canonical argument mapping")
            .1;
        let name = params.get(index)?;
        (!name.is_empty() && params.iter().rposition(|param| param == name) == Some(index))
            .then(|| (owner, name.clone()))
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn rebuild_mapped_argument_owners(
        &mut self,
    ) {
        self.mapped_argument_owners.clear();
        // Rebuild this projection once after scope restoration/remapping. The
        // index does not make otherwise unreachable invocation frames live.
        for (frame, mappings) in &self.mapped_arguments {
            for (id, params) in mappings {
                for name in params.iter().filter(|name| !name.is_empty()) {
                    let owner = self
                        .captured_bindings
                        .get(frame)
                        .and_then(|bindings| bindings.get(name))
                        .copied()
                        .unwrap_or(*frame);
                    self.mapped_argument_owners.entry(*id).or_insert(owner);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
