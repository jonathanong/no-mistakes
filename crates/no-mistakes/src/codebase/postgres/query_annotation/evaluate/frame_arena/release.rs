//! Stable identities allow deleting a dead frame's index edges without remapping peers.
use super::super::{Environment, Evaluator};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn release_frame(&mut self, env: Environment) {
        self.drop_frame_references(env);
        self.drop_frame_callback_keys(env);
        if let Some(origins) = self.captured_bindings.remove(&env) {
            for (name, origin) in origins {
                if let Some(names) = self.captured_binding_readers.get_mut(&origin) {
                    if let Some(readers) = names.get_mut(&name) {
                        readers.remove(&env);
                        if readers.is_empty() {
                            names.remove(&name);
                        }
                    }
                    if names.is_empty() {
                        self.captured_binding_readers.remove(&origin);
                    }
                }
            }
        }
        if let Some(mappings) = self.mapped_arguments.remove(&env) {
            for (id, _) in mappings {
                if let Some(readers) = self.fresh_mapped_argument_bindings.get_mut(&id) {
                    readers.remove(&env);
                    if readers.is_empty() {
                        self.fresh_mapped_argument_bindings.remove(&id);
                    }
                }
                if self.mapped_argument_owners.get(&id) == Some(&env) {
                    self.mapped_argument_owners.remove(&id);
                    self.mapped_parameter_indices.remove(&id);
                }
            }
        }
        self.fresh_mapped_parameters.remove(&env);
        self.scopes.release(env);
    }
}
