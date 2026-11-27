use crate::codebase::dependencies::graph::{GraphBuildPlan, GraphFiles};
use crate::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Builds the automatic catalog in the same frozen request universe used to
/// prepare runner projects. Keeping this shared prevents standalone reports
/// from missing aliases owned by configured runner or framework project roots.
pub(crate) struct FrameworkCatalogPreparation<'a> {
    pub(crate) root: &'a Path,
    pub(crate) tsconfig_path: Option<&'a Path>,
    pub(crate) tsconfig: &'a TsConfig,
    pub(crate) config: &'a crate::config::v2::NoMistakesConfig,
    pub(crate) codebase_config: &'a crate::codebase::config::Config,
    pub(crate) workspace:
        &'a std::sync::Arc<crate::codebase::workspace_resolution::IndexedWorkspaceMap>,
    pub(crate) root_visible_paths: &'a [PathBuf],
    pub(crate) visible_paths: &'a crate::codebase::ts_source::VisiblePathSnapshot,
    pub(crate) sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    pub(crate) build_plan: GraphBuildPlan,
    pub(crate) graph_files: &'a GraphFiles,
    pub(crate) collect_graph_facts: bool,
    pub(crate) framework_plan: &'a super::plan::FrameworkPreparationPlan,
}

pub(crate) type FrameworkCatalog = (Arc<TsConfigCatalog>, Box<dyn super::PreparedProjects>);
