//! Root selection shared with sibling call-graph rules.
use super::config::Options;
use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::config::v2::schema::StringOrList;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(crate) struct SharedRoots<'a> {
    pub rule: &'a str,
    pub root: &'a Path,
    /// `forbidden-calls` root selectors; `None` selects every Vitest project.
    pub roots: Option<&'a serde_yaml::Value>,
    pub exclude_roots: Option<&'a StringOrList>,
    pub graph: &'a DepGraph,
    pub vitest: Option<&'a crate::codebase::rules::PreparedVitestProjectCatalog>,
    pub graph_files: &'a [PathBuf],
    pub target_roots: &'a [PathBuf],
}

pub(crate) fn expand_roots(request: SharedRoots<'_>) -> Result<(Vec<NodeId>, Vec<PathBuf>)> {
    let mut mapping = serde_yaml::Mapping::new();
    let default_roots: serde_yaml::Value = serde_yaml::from_str("[{vitest: true}]")?;
    mapping.insert(
        "roots".into(),
        request.roots.cloned().unwrap_or(default_roots),
    );
    if let Some(exclude) = request.exclude_roots {
        mapping.insert("excludeRoots".into(), serde_yaml::to_value(exclude)?);
    }
    let options: Options = serde_yaml::from_value(mapping.into())
        .with_context(|| format!("{}: invalid roots", request.rule))?;
    super::config::validate(&options)?;
    super::roots::expand_with_files(super::roots::ExpandRequest {
        rule: request.rule,
        root: request.root,
        options: &options,
        graph: request.graph,
        vitest: request.vitest,
        playwright: None,
        graph_files: request.graph_files,
        target_roots: request.target_roots,
    })
}
