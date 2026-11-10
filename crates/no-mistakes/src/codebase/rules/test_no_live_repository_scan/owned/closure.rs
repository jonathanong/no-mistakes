//! Whether an unreadable module can reach an enumerating function from a test root.
use super::cause::{Cause, CauseKind};
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};
use std::path::{Path, PathBuf};

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
        let blocked = !self.blocking_sites(module).is_empty();
        self.reached
            .borrow_mut()
            .insert(module.to_path_buf(), blocked);
        blocked
    }

    pub(super) fn unread_cause(
        &self,
        module: &Path,
        source: &NodeId,
        facts: &dyn TsFactLookup,
    ) -> Cause {
        let sites = self.blocking_sites(module);
        let Some((file, line)) = sites.first().cloned() else {
            return super::route_cause::function_fallback(&self.callers, source, facts, None);
        };
        let extra = sites.len().saturating_sub(1) as u32;
        let reached = super::closure_path::reach_path(
            self.graph,
            &self.gate_roots,
            &file,
            self.setups.root(),
        );
        Cause::new(CauseKind::UnreadModule, file, line, "")
            .with_extra(extra)
            .with_reached(reached)
    }

    fn blocking_sites(&self, module: &Path) -> Vec<(PathBuf, u32)> {
        let closure = self.reaching_closure(module);
        let mut sites = Vec::new();
        for (file, line) in self
            .escaped
            .unknown_sites()
            .iter()
            .chain(self.consumed.unknown_sites())
        {
            if closure.contains(file) {
                sites.push((file.clone(), *line));
            }
        }
        sites.sort();
        sites.dedup();
        sites.sort_by(|left, right| {
            super::closure_path::hops(self.graph, &self.gate_roots, &right.0)
                .cmp(&super::closure_path::hops(
                    self.graph,
                    &self.gate_roots,
                    &left.0,
                ))
                .then(left.cmp(right))
        });
        sites
    }

    fn reaching_closure(&self, module: &Path) -> crate::fx::FxHashSet<PathBuf> {
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
    }
}

#[cfg(test)]
#[path = "closure/tests.rs"]
mod tests;
