use super::{build_graph, ReversePrepared, Target};
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::ts_source::facts::TsFactMap;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

/// Ordinary query projections owned by an aggregate request. Runner catalogs
/// and explicitly promoted sibling targets have different public semantics.
pub(crate) struct OrdinaryCallSitesPlan {
    targets: Vec<Target>,
    catalog: ReversePrepared,
    projections: Vec<(usize, GraphFiles)>,
    projection_by_target: FxHashMap<PathBuf, usize>,
}

pub(crate) struct OrdinaryCallSites {
    projections: Vec<(GraphFiles, std::sync::Arc<DepGraph>)>,
    projection_by_target: FxHashMap<PathBuf, usize>,
}

impl OrdinaryCallSitesPlan {
    pub(crate) fn new(
        files: &[PathBuf],
        root: &Path,
        tsconfig: Option<&Path>,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> anyhow::Result<Self> {
        let targets = super::super::super::shared::resolve_targets_with_session(
            files,
            Some(root),
            tsconfig,
            session,
        )?;
        let catalog = targets[0].prepare_reverse_base()?;
        let mut by_universe = FxHashMap::default();
        let mut projection_by_target = FxHashMap::default();
        let mut projections = Vec::new();
        for (target_index, target) in targets.iter().enumerate() {
            let extra = (!catalog.graph_files.contains_visible(&target.abs_file))
                .then(|| target.abs_file.clone());
            let projection = *by_universe.entry(extra.clone()).or_insert_with(|| {
                let mut files = catalog
                    .graph_files
                    .visible_subset(catalog.graph_files.all().to_vec());
                if let Some(extra) = &extra {
                    files.add_explicit_root(extra);
                }
                let index = projections.len();
                projections.push((target_index, files));
                index
            });
            projection_by_target.insert(target.abs_file.clone(), projection);
        }
        Ok(Self {
            targets,
            catalog,
            projections,
            projection_by_target,
        })
    }

    pub(crate) fn files(&self) -> Vec<PathBuf> {
        let mut files = self
            .projections
            .iter()
            .flat_map(|(_, files)| files.indexable().iter().cloned())
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        files.sort();
        files
    }

    pub(crate) fn materialize(
        self,
        facts: &TsFactMap,
        reuse: Option<(
            &GraphFiles,
            &crate::codebase::ts_resolver::TsConfigCatalog,
            std::sync::Arc<DepGraph>,
        )>,
    ) -> anyhow::Result<OrdinaryCallSites> {
        let projections = self
            .projections
            .into_iter()
            .map(|(target, files)| {
                // Call resolution depends on the universe, each importing file's
                // selected config, and the request-shared workspace/facts. A
                // broader runner catalog is reusable only when those agree.
                let shared = reuse.as_ref().filter(|(shared_files, catalog, _)| {
                    files.all() == shared_files.all()
                        && files.indexable() == shared_files.indexable()
                        && files.iter_visible().eq(shared_files.iter_visible())
                        && files.indexable().iter().all(|path| {
                            self.catalog.tsconfig_catalog.config_for(path)
                                == catalog.config_for(path)
                        })
                });
                let graph = if let Some((_, _, graph)) = shared {
                    self.targets[target].session.record_work("graph.reuses", 1);
                    graph.clone()
                } else {
                    std::sync::Arc::new(build_graph(
                        &self.targets[target],
                        &files,
                        &self.catalog.tsconfig_catalog,
                        self.catalog.workspace.clone(),
                        facts,
                    )?)
                };
                Ok((files, graph))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(OrdinaryCallSites {
            projections,
            projection_by_target: self.projection_by_target,
        })
    }
}

impl OrdinaryCallSites {
    pub(crate) fn projection_for(&self, file: &Path) -> Option<(&GraphFiles, &DepGraph)> {
        let index = self.projection_by_target.get(file)?;
        let (files, graph) = &self.projections[*index];
        Some((files, graph))
    }
}
