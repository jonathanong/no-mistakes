//! Root selection shared with sibling call-graph rules.
use super::root::{validate_root_options, Root};
use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::config::v2::schema::StringOrList;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct RootOptions {
    roots: Vec<Root>,
    exclude_roots: Option<StringOrList>,
}

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
    let options: RootOptions = serde_yaml::from_value(mapping.into())
        .with_context(|| format!("{}: invalid roots", request.rule))?;
    validate_root_options(&options.roots, options.exclude_roots.as_ref())?;
    super::roots::expand_with_files(super::roots::ExpandRequest {
        rule: request.rule,
        allow_sourceless: true,
        root: request.root,
        roots: &options.roots,
        exclude_roots: options.exclude_roots.as_ref(),
        graph: request.graph,
        vitest: request.vitest,
        playwright: None,
        graph_files: request.graph_files,
        target_roots: request.target_roots,
    })
}
