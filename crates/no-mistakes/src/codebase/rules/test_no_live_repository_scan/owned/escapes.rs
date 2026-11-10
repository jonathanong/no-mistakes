use super::path_provenance;
use super::route_index;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::module_facts::arguments::RouteKind;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct Escapes {
    local: FxHashMap<PathBuf, FxHashSet<String>>,
    refs: Vec<(PathBuf, u32, String)>,
    unknown: bool,
    unknown_at: Option<(PathBuf, u32)>,
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
                    if paths.modules_unknown {
                        escapes.unknown = true;
                        route_index::note_escape(
                            &mut escapes.unknown_at,
                            file,
                            paths.unknown_load.unwrap_or(0),
                        );
                    }
                    for site in &paths.route_sites {
                        if site.kind == RouteKind::NonDirect {
                            escapes
                                .refs
                                .push((file.clone(), site.start, site.detail.clone()));
                        }
                    }
                    escapes.local.insert(file.clone(), paths.value_refs.clone());
                }
                None => {
                    if ts.module_bindings.is_some() {
                        escapes.unknown = true;
                        route_index::note_escape(&mut escapes.unknown_at, file, 0);
                    }
                }
            }
        }
        escapes
    }

    pub(super) fn unknown(&self) -> bool {
        self.unknown
    }

    pub(super) fn unknown_at(&self) -> Option<&(PathBuf, u32)> {
        self.unknown_at.as_ref()
    }

    pub(super) fn contains(&self, file: &Path, name: &str) -> bool {
        let short = name.rsplit('.').next().unwrap_or(name);
        self.local
            .get(file)
            .is_some_and(|names| names.contains(name) || names.contains(short))
    }

    pub(super) fn reference(&self, file: &Path, name: &str) -> Option<(PathBuf, u32, String)> {
        let short = name.rsplit('.').next().unwrap_or(name);
        self.refs
            .iter()
            .filter(|(path, _, detail)| path == file && (detail == name || detail == short))
            .min_by_key(|(_, start, _)| *start)
            .map(|(path, start, detail)| (path.clone(), *start, detail.clone()))
    }
}
