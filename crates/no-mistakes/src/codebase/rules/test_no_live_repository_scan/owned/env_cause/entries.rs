use super::{Entry, Owners};
use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use std::path::Path;

pub(super) fn diagnostic_paths(facts: &TsFileFacts) -> Option<&PathProvenance> {
    if facts.operational_error.is_some() || facts.parse_error.is_some() || facts.fatal_parse_error {
        return None;
    }
    facts.module_bindings.as_deref()?.path_provenance.as_deref()
}

impl Owners<'_> {
    pub(super) fn environment_entries(
        &self,
        module: &Path,
        facts: &dyn TsFactLookup,
    ) -> Vec<Entry> {
        if let Some(entries) = self.env_entries.borrow().get(module) {
            return entries.clone();
        }
        let mut entries = self
            .consumed
            .targeting(module)
            .into_iter()
            .filter(|hit| self.gate_roots.contains(&hit.file))
            .map(|hit| Entry {
                file: hit.file.clone(),
                offset: Some(hit.start),
                line: hit.line,
                concrete: true,
            })
            .collect::<Vec<_>>();
        if entries.is_empty() {
            let kinds = super::super::super::reachable::load_kinds();
            let mut reaching = self
                .graph
                .dependents_of(&[NodeId::file(module)], None, Some(&kinds))
                .into_iter()
                .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
                .collect::<Vec<_>>();
            reaching.push(module.to_path_buf());
            for file in self
                .gate_roots
                .iter()
                .filter(|file| reaching.contains(file))
            {
                let neighbors = self
                    .graph
                    .dependencies_of_node(&NodeId::file(file))
                    .into_iter()
                    .flatten()
                    .filter(|(node, kind)| {
                        kinds.contains(kind)
                            && node
                                .as_file()
                                .is_some_and(|next| reaching.iter().any(|path| path == next))
                    })
                    .map(|(node, _)| node)
                    .collect::<crate::fx::FxHashSet<_>>();
                let Some(ts) = facts.get_ts_facts(file) else {
                    continue;
                };
                let Some(paths) = diagnostic_paths(ts) else {
                    continue;
                };
                let imports = ts
                    .imports
                    .iter()
                    .filter(|import| {
                        !matches!(
                            import.kind,
                            crate::codebase::dependencies::extract::ImportKind::Type
                        ) && import.specifier.starts_with('.')
                    })
                    .collect::<Vec<_>>();
                let names = imports
                    .iter()
                    .map(|import| &import.specifier)
                    .collect::<crate::fx::FxHashSet<_>>();
                if neighbors.len() == 1 && names.len() == 1 {
                    let name = *names.iter().next().expect("one import name");
                    let routes = paths
                        .route_sites
                        .iter()
                        .filter(|route| route.detail == *name)
                        .collect::<Vec<_>>();
                    if !routes.is_empty() {
                        entries.extend(routes.into_iter().map(|route| Entry {
                            file: file.clone(),
                            offset: Some(route.start),
                            line: route.line,
                            concrete: true,
                        }));
                        continue;
                    }
                    if let Some(import) = imports.first() {
                        entries.push(Entry {
                            file: file.clone(),
                            offset: None,
                            line: import.line,
                            concrete: true,
                        });
                        continue;
                    }
                }
                let line = paths
                    .test_callbacks
                    .iter()
                    .map(|(start, _)| super::super::cause::line_at(facts, file, *start))
                    .min()
                    .unwrap_or(1);
                entries.push(Entry {
                    file: file.clone(),
                    offset: None,
                    line,
                    concrete: false,
                });
            }
        }
        self.env_entries
            .borrow_mut()
            .insert(module.to_path_buf(), entries.clone());
        entries
    }
}
