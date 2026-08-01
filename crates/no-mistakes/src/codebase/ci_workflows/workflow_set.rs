//! Request-scoped GitHub Actions workflow documents.
//!
//! CI consumers deliberately share this raw YAML layer rather than each
//! discovering, reading, and deserializing workflow files for themselves.
//! Consumers retain ownership of their output-specific diagnostics: `ci
//! topology` coalesces load failures into malformed-workflow diagnostics while
//! `ci impact` and `ci env` distinguish read and parse warnings.

use crate::codebase::ci_graph::{discover_workflow_files_from_snapshot, relative_slash};
use crate::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use crate::config::v2::schema::CiConfig;
use rayon::prelude::*;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A workflow YAML load failure retained for consumer-specific rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDocumentError {
    pub kind: WorkflowDocumentErrorKind,
    pub message: String,
}

/// Whether loading failed before or during YAML deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowDocumentErrorKind {
    Read,
    Parse,
}

/// One discovered workflow path and its parsed YAML document.
#[derive(Debug, Clone)]
pub struct ParsedWorkflowDocument {
    /// Repository-relative, slash-normalized path (or the normalized external
    /// path when a standalone CI command explicitly configures one).
    pub path: String,
    /// Raw GitHub Actions YAML, parsed exactly once for this request.
    pub value: Result<Value, WorkflowDocumentError>,
}

/// All configured workflow documents for one invocation, sorted by path.
#[derive(Debug, Clone)]
pub struct ParsedWorkflowSet {
    pub documents: Vec<ParsedWorkflowDocument>,
}

impl ParsedWorkflowSet {
    /// Discover, read, and deserialize the configured workflows once.
    pub fn load(root: &Path, ci: &CiConfig) -> Self {
        let snapshot = VisiblePathSnapshot::new(root);
        Self::load_from_snapshot(root, ci, &snapshot)
    }

    /// Reuses an invocation's visibility snapshot for workflow discovery.
    #[doc(hidden)]
    pub fn load_from_snapshot(root: &Path, ci: &CiConfig, snapshot: &VisiblePathSnapshot) -> Self {
        let sources = snapshot.source_store_for(root);
        Self::load_from_snapshot_and_sources(root, ci, snapshot, &sources)
    }

    /// Reuses both an invocation's visibility snapshot and canonical source store.
    #[doc(hidden)]
    pub fn load_from_snapshot_and_sources(
        root: &Path,
        ci: &CiConfig,
        snapshot: &VisiblePathSnapshot,
        sources: &SourceStore,
    ) -> Self {
        Self::from_paths_and_sources(
            root,
            discover_workflow_files_from_snapshot(root, ci, snapshot),
            sources,
        )
    }

    /// Parse a caller-provided workflow universe without rediscovering files.
    ///
    /// The dependency graph uses this constructor after filtering its already
    /// discovered file universe to `ci.workflow_dirs`, so it keeps the
    /// repository's one-discovery-pass invariant.
    pub fn from_paths(root: &Path, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        let paths: Vec<PathBuf> = paths
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let snapshot = VisiblePathSnapshot::from_paths(root, &paths);
        let sources = snapshot.source_store_for(root);
        Self::from_paths_and_sources(root, paths, &sources)
    }

    /// Parse a caller-provided workflow universe through prepared request sources.
    #[doc(hidden)]
    pub fn from_paths_and_sources(
        root: &Path,
        paths: impl IntoIterator<Item = PathBuf>,
        sources: &SourceStore,
    ) -> Self {
        let paths: Vec<PathBuf> = paths
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let documents = paths
            .into_par_iter()
            .map(|absolute| {
                let path = relative_slash(root, &absolute);
                let value = sources
                    .read_path(&absolute)
                    .map_err(|error| WorkflowDocumentError {
                        kind: WorkflowDocumentErrorKind::Read,
                        message: error.to_string(),
                    })
                    .and_then(|source| {
                        serde_yaml::from_str(&source).map_err(|error| WorkflowDocumentError {
                            kind: WorkflowDocumentErrorKind::Parse,
                            message: error.to_string(),
                        })
                    });
                ParsedWorkflowDocument { path, value }
            })
            .collect::<Vec<_>>();

        let mut documents = documents;
        documents.sort_by(|left, right| left.path.cmp(&right.path));
        Self { documents }
    }

    /// Project already-parsed documents onto an output-specific path universe.
    #[doc(hidden)]
    pub fn project_paths(&self, root: &Path, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        let selected = paths
            .into_iter()
            .map(|path| relative_slash(root, &path))
            .collect::<BTreeSet<_>>();
        Self {
            documents: self
                .documents
                .iter()
                .filter(|document| selected.contains(&document.path))
                .cloned()
                .collect(),
        }
    }
}
