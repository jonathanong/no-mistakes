//! Whether an unreadable module can reach an enumerating function from a test root.
use super::Owners;
use crate::codebase::dependencies::graph::NodeId;
use std::path::Path;

impl Owners<'_> {
    /// True when caller proof must stop. Unknown test roots block every function.
    /// Otherwise only unreadable modules in this function's test-root closure do.
    pub(super) fn closure_unknown(&self, module: &Path) -> bool {
        if !self.setups.known() {
            return true;
        }
        if let Some(blocked) = self.reached.borrow().get(module).copied() {
            return blocked;
        }
        let blocked = self.reaches_unreadable(module);
        self.reached
            .borrow_mut()
            .insert(module.to_path_buf(), blocked);
        blocked
    }

    fn reaches_unreadable(&self, module: &Path) -> bool {
        if self.unknown_files.is_empty() {
            return false;
        }
        let mut seeds = Vec::new();
        if self.test_closure.contains(module) {
            seeds.push(module.to_path_buf());
        }
        let kinds = super::super::reachable::load_kinds();
        for entry in self
            .graph
            .dependents_of(&[NodeId::file(module)], None, Some(&kinds))
        {
            let Some(file) = entry.node.as_file() else {
                continue;
            };
            if self.test_closure.contains(file) {
                seeds.push(file.to_path_buf());
            }
        }
        super::super::reachable::module_files(self.graph, &seeds)
            .iter()
            .any(|file| self.unknown_files.contains(file))
    }
}
