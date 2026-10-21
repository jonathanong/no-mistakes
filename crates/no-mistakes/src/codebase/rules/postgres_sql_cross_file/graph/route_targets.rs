use super::resolve::*;
use super::route_index::RegistryRoutes;
use crate::codebase::dependencies::extract::ImportKind;
use crate::fx::fx_set;

impl RegistryRoutes<'_> {
    pub(super) fn target(
        &self,
        graph: &DepGraph,
        file: &Path,
        specifier: &str,
        kind: ImportKind,
    ) -> Option<PathBuf> {
        let index = usize::from(kind == ImportKind::Require);
        let projection =
            self.file(file)?.targets[index].get_or_init(|| self.project_target(graph, file, kind));
        projection
            .as_ref()
            .filter(|(actual, _)| actual == specifier)
            .map(|(_, target)| target.clone())
    }
    /// Canonical edges do not retain specifiers: require one internal path and
    /// one target after excluding proven external module nodes.
    fn project_target(
        &self,
        graph: &DepGraph,
        file: &Path,
        kind: ImportKind,
    ) -> Option<(String, PathBuf)> {
        self.record(2);
        let edge = if kind == ImportKind::Require {
            EdgeKind::Require
        } else {
            EdgeKind::Import
        };
        let neighbors = graph.dependencies_of_node(&NodeId::file(file))?;
        let mut external = fx_set();
        let mut files = fx_set();
        for (target, actual) in neighbors {
            self.record(1);
            if *actual != edge {
                continue;
            }
            match target {
                NodeId::Module(name) => {
                    external.insert(&**name);
                }
                _ => {
                    if let Some(path) = target.as_file() {
                        files.insert(path);
                    }
                }
            }
        }
        let ts = self.facts.get_ts_facts(file)?;
        let mut specifiers = fx_set();
        for import in &ts.imports {
            self.record(1);
            if import.kind != kind {
                continue;
            }
            if import.computed {
                return None;
            }
            if !external.contains(import.specifier.as_str()) {
                specifiers.insert(import.specifier.as_str());
            }
        }
        if specifiers.len() != 1 || files.len() != 1 {
            return None;
        }
        Some((
            specifiers.into_iter().next()?.to_string(),
            files.into_iter().next()?.to_path_buf(),
        ))
    }
}
